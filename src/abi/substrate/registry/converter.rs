use std::{any::TypeId, collections::HashMap, sync::Mutex};

use ink_metadata::{
    layout::{CellLayout, FieldLayout, Layout, LayoutKey, StructLayout},
    ConstructorSpec, ContractSpec, EventParamSpec, EventSpec, InkProject, MessageParamSpec,
    MessageSpec, ReturnTypeSpec, Selector, TypeSpec,
};
use ink_primitives::Key;
use once_cell::sync::{Lazy, OnceCell};
use scale_info::{
    form::PortableForm, interner::UntrackedSymbol, MetaType, Path, PortableRegistry, Registry,
    Type, TypeDef, TypeDefArray, TypeDefComposite, TypeDefPrimitive, TypeDefSequence,
    TypeDefVariant, TypeParameter,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::abi::substrate::{Abi, Array, ArrayDef, Composite, EnumDef, SequenceDef};

use super::string_to_static_str;

#[derive(Serialize, Deserialize, Clone)]
pub struct RPortableType {
    id: u32,
    #[serde(rename = "type")]
    ty: RType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RType {
    /// The unique path to the type. Can be empty for built-in types
    #[serde(skip_serializing_if = "Path::is_empty", default)]
    path: Path<PortableForm>,
    /// The generic type parameters of the type in use. Empty for non generic types
    #[serde(rename = "params", skip_serializing_if = "Vec::is_empty", default)]
    type_params: Vec<TypeParameter<PortableForm>>,
    /// The actual type definition
    #[serde(rename = "def")]
    type_def: TypeDef<PortableForm>,
    /// Documentation
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    docs: Vec<String>,
}

fn primitive_to_typedef(abi: &Abi, def: &super::super::PrimitiveDef) -> TypeDef<PortableForm> {
    let def = match def.primitive.as_str() {
        "u8" => TypeDefPrimitive::U8,
        "u16" => TypeDefPrimitive::U16,
        "u32" => TypeDefPrimitive::U32,
        "u64" => TypeDefPrimitive::U64,
        "u128" => TypeDefPrimitive::U128,
        "u256" => TypeDefPrimitive::U256,
        "i8" => TypeDefPrimitive::I8,
        "i16" => TypeDefPrimitive::I16,
        "i32" => TypeDefPrimitive::I32,
        "i64" => TypeDefPrimitive::I64,
        "i128" => TypeDefPrimitive::I128,
        "i256" => TypeDefPrimitive::I256,
        "bool" => TypeDefPrimitive::Bool,
        "str" => TypeDefPrimitive::Str,
        "AccountId" => {
            let (idx, _) = abi
                .types
                .iter()
                .enumerate()
                .find(|(idx, e)| {
                    if let crate::abi::substrate::Type::Builtin { def } = e {
                        def.primitive == "u8"
                    } else {
                        false
                    }
                })
                .unwrap();

            let arr_def = ArrayDef {
                array: Array { len: 32, ty: idx },
            };

            return TypeDef::Array(array_to_typedef(&arr_def));
        }
        _ => {
            unimplemented!()
        }
    };

    TypeDef::Primitive(def)
}

fn array_to_typedef(def: &super::super::ArrayDef) -> TypeDefArray<PortableForm> {
    let arr_json = json!({
        "len": def.array.len as u32,
        "type": def.array.ty as i32
    });

    let def: TypeDefArray<PortableForm> = serde_json::from_value(arr_json).unwrap();
    def
}

fn sequence_to_typedef(def: &SequenceDef) -> TypeDefSequence<PortableForm> {
    let arr_json = json!({ "type": def.sequence.ty as i32 });

    let def: TypeDefSequence<PortableForm> = serde_json::from_value(arr_json).unwrap();
    def
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RField {
    /// The name of the field. None for unnamed fields.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    name: Option<String>,
    /// The type of the field.
    #[serde(rename = "type")]
    ty: i32,
    /// The name of the type of the field as it appears in the source code.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    type_name: Option<String>,
    /// Documentation
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    docs: Vec<String>,
}

fn composite_to_typedef(def: &Composite) -> TypeDefComposite<PortableForm> {
    let mut fields: Vec<RField> = vec![];

    for f in &def.composite.fields {
        let rf = RField {
            name: f.name.clone(),
            ty: f.ty as i32,
            type_name: None,
            docs: vec![],
        };

        fields.push(rf);
    }

    let def_json = json!({ "fields": fields });

    serde_json::from_value(def_json).unwrap()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RVariant {
    /// The name of the variant.
    name: String,
    /// The fields of the variant.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    fields: Vec<RField>,
    /// Index of the variant, used in `parity-scale-codec`.
    ///
    /// The value of this will be, in order of precedence:
    ///     1. The explicit index defined by a `#[codec(index = N)]` attribute.
    ///     2. The implicit index from the position of the variant in the `enum` definition.
    index: u8,
    /// Documentation
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    docs: Vec<String>,
}

fn enum_to_typedef(def: &EnumDef) -> TypeDefVariant<PortableForm> {
    let mut variants: Vec<RVariant> = vec![];

    for v in &def.variant.variants {
        let rv = RVariant {
            name: v.name.clone(),
            fields: vec![],
            index: v.discriminant as u8,
            docs: vec![v.name.clone()],
        };

        variants.push(rv);
    }

    let def_json = json!({ "variants": variants });

    serde_json::from_value(def_json).unwrap()
}

static TYPEMAP: Lazy<Mutex<HashMap<i32, RPortableType>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn resolve_type(abi: &Abi, abi_id: i32) -> RType {
    let handle = TYPEMAP.lock().unwrap();

    match handle.get(&abi_id) {
        Some(r) => r.ty.clone(),
        None => {
            drop(handle);

            let ty = abi.types.get(abi_id as usize - 1).unwrap();
            let rtype = match ty {
                crate::abi::substrate::Type::Builtin { def } => {
                    let type_def = primitive_to_typedef(&abi, def);

                    let t = RType {
                        path: Default::default(),
                        type_params: vec![],
                        type_def,
                        docs: vec![],
                    };

                    let rtype = RPortableType {
                        id: abi_id as u32,
                        ty: t,
                    };

                    rtype
                }
                crate::abi::substrate::Type::BuiltinArray { def } => {
                    let typedef = array_to_typedef(def);

                    let t = RType {
                        path: Default::default(),
                        type_params: vec![],
                        type_def: TypeDef::Array(typedef),
                        docs: vec![],
                    };

                    let rtype = RPortableType {
                        id: abi_id as u32,
                        ty: t,
                    };

                    rtype
                }
                crate::abi::substrate::Type::BuiltinSequence { def } => {
                    let typedef = sequence_to_typedef(def);

                    let t = RType {
                        path: Default::default(),
                        type_params: vec![],
                        type_def: TypeDef::Sequence(typedef),
                        docs: vec![],
                    };

                    let rtype = RPortableType {
                        id: abi_id as u32,
                        ty: t,
                    };

                    rtype
                }
                crate::abi::substrate::Type::Struct { path, def } => {
                    let typedef = composite_to_typedef(def);

                    let p = if path.is_empty() {
                        Path::<PortableForm>::default()
                    } else {
                        let path_json = json!(path);
                        serde_json::from_value::<Path<PortableForm>>(path_json).unwrap()
                    };

                    let t = RType {
                        path: p,
                        type_params: vec![],
                        type_def: TypeDef::Composite(typedef),
                        docs: vec![],
                    };

                    let rtype = RPortableType {
                        id: abi_id as u32,
                        ty: t,
                    };

                    rtype
                }
                crate::abi::substrate::Type::Enum { path, def } => {
                    let typedef = enum_to_typedef(def);

                    let path = if path.is_empty() {
                        let path_json = json!(path);
                        serde_json::from_value(path_json).unwrap()
                    } else {
                        let path_json = json!([format!("Enum{abi_id}")]);
                        serde_json::from_value(path_json).unwrap()
                    };
                    println!("{path:?}");

                    let t = RType {
                        path,
                        type_params: vec![],
                        type_def: TypeDef::Variant(typedef),
                        docs: vec![],
                    };

                    let rtype = RPortableType {
                        id: abi_id as u32,
                        ty: t,
                    };

                    rtype
                }
            };

            {
                let mut handle = TYPEMAP.lock().unwrap();
                handle.insert(abi_id as i32, rtype.clone());
            }

            rtype.ty
        }
    }
}

pub fn abi_to_types(abi: &Abi) -> PortableRegistry {
    let mut types = vec![];
    for (idx, _) in abi.types.iter().enumerate() {
        let ty = resolve_type(&abi, idx as i32 + 1);

        let rty = RPortableType {
            id: idx as u32 + 1,
            ty,
        };

        types.push(rty);
    }

    let registry_json = json!({ "types": types });

    serde_json::from_value(registry_json).unwrap()
}

#[derive(Serialize, Deserialize)]
pub struct RCellLayout {
    key: LayoutKey,
    ty: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RFieldLayout {
    /// The name of the field.
    ///
    /// Can be missing, e.g. in case of an enum tuple struct variant.
    name: Option<String>,
    /// The kind of the field.
    ///
    /// This is either a direct layout bound
    /// or another recursive layout sub-struct.
    layout: Layout<PortableForm>,
}

use hex::FromHex;

pub fn abi_to_layout(abi: &Abi) -> Layout<PortableForm> {
    let root = &abi.storage.structs;

    let mut fields: Vec<RFieldLayout> = vec![];

    for s in root.fields.iter() {
        let name = &s.name;

        let key_buffer = <[u8; 32]>::from_hex(&s.layout.cell.key[2..]).unwrap(); // SKIP 0X
        let key = Key::new(key_buffer);
        let layout_key = LayoutKey::from(key);

        let ty = &s.layout.cell.ty;
        let cell_json = json!({ "ty": ty, "key": layout_key });

        let cell = serde_json::from_value::<CellLayout<PortableForm>>(cell_json).unwrap();
        let layout = Layout::Cell(cell);

        let flayout = RFieldLayout {
            name: Some(name.clone()),
            layout,
        };

        fields.push(flayout);
    }

    let s_json = json!({ "fields": fields });

    let s = serde_json::from_value::<StructLayout<PortableForm>>(s_json).unwrap();

    Layout::Struct(s)
}

pub fn abi_to_spec(abi: &Abi, project: &InkProject) -> ContractSpec<PortableForm> {
    let constructors = abi_to_constructors(abi, project);
    let msgs = abi_to_msgs(abi, project);
    let evts = abi_to_evts(abi, project);
    let docs = Vec::<String>::new();

    let spec_json = json!({
        "constructors": constructors,
        "messages": msgs,
        "events": evts,
        "docs": docs
    });

    serde_json::from_value(spec_json).unwrap()
}

pub fn abi_to_constructors(abi: &Abi, project: &InkProject) -> Vec<ConstructorSpec<PortableForm>> {
    let spec = &abi.spec;

    let mut out = vec![];

    for (c_in, c) in spec
        .constructors
        .iter()
        .zip(project.spec().constructors().iter())
    {
        let mut args = vec![];

        for (c_in_arg, c_arg) in c_in.args.iter().zip(c.args().iter()) {
            let spec_json = json!({
                "type": c_in_arg.ty.ty,
                "displayName": c_arg.ty().display_name()
            });

            let spec = serde_json::from_value::<TypeSpec<PortableForm>>(spec_json).unwrap();

            let value = json!({
                "label": c_arg.label(),
                "type": spec
            });

            let m_arg_out =
                serde_json::from_value::<MessageParamSpec<PortableForm>>(value).unwrap();

            args.push(m_arg_out);
        }

        let value = json!( {
            "label": c.label(),
            "selector": c.selector(),
            "payable": c.payable(),
            "args": args,
            "docs": c.docs(),
        });

        let c_out = serde_json::from_value::<ConstructorSpec<PortableForm>>(value).unwrap();
        out.push(c_out);
    }

    out
}

pub fn abi_to_msgs(abi: &Abi, project: &InkProject) -> Vec<MessageSpec<PortableForm>> {
    let mut out = vec![];

    for (m_in, m) in abi
        .spec
        .messages
        .iter()
        .zip(project.spec().messages().iter())
    {
        let mut args = vec![];

        for (m_in_arg, m_arg) in m_in.args.iter().zip(m.args().iter()) {
            let spec_json = json!({
                "type": m_in_arg.ty.ty,
                "displayName": m_arg.ty().display_name()
            });

            let spec = serde_json::from_value::<TypeSpec<PortableForm>>(spec_json).unwrap();

            let value = json!({
                "label": m_arg.label(),
                "type": spec
            });

            let m_arg_out =
                serde_json::from_value::<MessageParamSpec<PortableForm>>(value).unwrap();

            args.push(m_arg_out);
        }

        println!("{:?}", m_in.return_type.as_ref().map(|e| e.ty));

        let inner = if let Some(rt) = &m_in.return_type {
            let spec_json = json!({
                "type": rt.ty,
                "displayName": rt.display_name
            });

            let spec = serde_json::from_value::<TypeSpec<PortableForm>>(spec_json).unwrap();
            Some(spec)
        } else {
            None
        };

        let value = json!(inner);

        let ret_type = serde_json::from_value::<ReturnTypeSpec<PortableForm>>(value).unwrap();

        let value = json!({
            "label": m.label(),
            "selector": m.selector(),
            "mutates": m.mutates(),
            "payable": m.payable(),
            "args": args,
            "returnType": ret_type,
            "docs": m.docs(),
        });

        let m_out = serde_json::from_value::<MessageSpec<PortableForm>>(value).unwrap();
        out.push(m_out);
    }

    out
}

pub fn abi_to_evts(abi: &Abi, project: &InkProject) -> Vec<EventSpec<PortableForm>> {
    let mut out = vec![];

    for (e_in, e) in abi.spec.events.iter().zip(project.spec().events().iter()) {
        let mut args = vec![];

        for (e_in_arg, e_arg) in e_in.args.iter().zip(e.args().iter()) {
            let spec_json = json!({
                "type": e_in_arg.param.ty.ty,
                "displayName": e_arg.ty().display_name()
            });

            let spec = serde_json::from_value::<TypeSpec<PortableForm>>(spec_json).unwrap();

            let value = json!({
                "indexed": e_arg.indexed(),
                "label": e_arg.label(),
                "type": spec,
                "docs": e_arg.docs()
            });

            let e_arg_out = serde_json::from_value::<EventParamSpec<PortableForm>>(value).unwrap();

            args.push(e_arg_out);
        }

        let value = json!({
            "label": e.label(),
            "args": args,
            "docs": e.docs(),
        });

        let e_out = serde_json::from_value::<EventSpec<PortableForm>>(value).unwrap();
        out.push(e_out);
    }

    out
}
