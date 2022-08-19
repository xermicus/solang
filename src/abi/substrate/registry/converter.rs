use std::{collections::HashMap, sync::Mutex};

use ink_metadata::{
    layout::{CellLayout, FieldLayout, Layout, LayoutKey, StructLayout},
    ConstructorSpec, ContractSpec, EventParamSpec, EventSpec, MessageParamSpec, MessageSpec,
    ReturnTypeSpec, TypeSpec,
};
use ink_primitives::Key;
use once_cell::sync::Lazy;
use scale_info::{
    form::PortableForm, Field, PortableRegistry, Type, TypeDef, TypeDefArray, TypeDefComposite,
    TypeDefPrimitive, TypeDefSequence, TypeDefVariant, Variant,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::abi::substrate::{
    Abi, Array, ArrayDef, Composite, Constructor, EnumDef, EnumVariant, Event, Message, Param,
    ParamIndexed, PrimitiveDef, SequenceDef, StorageStruct, StructField,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct RPortableType {
    /// idx within Abi.types
    id: usize,
    #[serde(rename = "type")]
    ty: Type<PortableForm>,
}

pub fn setup_cache(abi: &Abi) {
    let u8_idx = abi
        .types
        .iter()
        .enumerate()
        .find_map(|(idx, v)| {
            if let super::super::Type::Builtin { def } = v {
                if def.primitive == "u8" {
                    Some(idx)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();

    let abi_id = u8_idx as usize + 1;
    resolve_type(abi, abi_id);
}

fn get_u8_from_cache() -> usize {
    let handle = TYPEMAP.lock().unwrap();

    let idx = handle
        .iter()
        .find_map(|(k, v)| {
            if let TypeDef::Primitive(TypeDefPrimitive::U8) = v.ty.type_def() {
                Some(k)
            } else {
                None
            }
        })
        .unwrap();

    *idx
}

impl SerdeConversion for PrimitiveDef {
    type Output = TypeDef<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let def = match self.primitive.as_str() {
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
                // NOTICE: make sure u8 is already in cache
                let idx: usize = get_u8_from_cache();
                let arr_def = ArrayDef {
                    array: Array { len: 32, ty: idx },
                };
                return TypeDef::Array(arr_def.serde_cast());
            }
            _ => {
                unimplemented!()
            }
        };

        TypeDef::Primitive(def)
    }
}

impl SerdeConversion for ArrayDef {
    type Output = TypeDefArray<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let arr_json = json!({
            "len": self.array.len as u32,
            "type": self.array.ty as i32
        });

        let def: TypeDefArray<PortableForm> = serde_json::from_value(arr_json).unwrap();
        def
    }
}

impl SerdeConversion for SequenceDef {
    type Output = TypeDefSequence<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let seq_json = json!({ "type": self.sequence.ty as i32 });

        let def: TypeDefSequence<PortableForm> = serde_json::from_value(seq_json).unwrap();
        def
    }
}

impl SerdeConversion for StructField {
    type Output = Field<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let rf_json = json!({
            "name": self.name.clone(),
            "type": self.ty,
        });
        serde_json::from_value(rf_json).unwrap()
    }
}

impl SerdeConversion for Composite {
    type Output = TypeDefComposite<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let fields = self
            .composite
            .fields
            .iter()
            .map(|v| v.serde_cast())
            .collect::<Vec<_>>();

        let def_json = json!({ "fields": fields });

        serde_json::from_value(def_json).unwrap()
    }
}

impl SerdeConversion for EnumVariant {
    type Output = Variant<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let v_json = json!({
            "name": self.name,
            "index": self.discriminant as u8,
            "docs": vec![self.name.clone()],
        });

        serde_json::from_value(v_json).unwrap()
    }
}

impl SerdeConversion for EnumDef {
    type Output = TypeDefVariant<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let variants = self
            .variant
            .variants
            .iter()
            .map(|v| v.serde_cast())
            .collect::<Vec<_>>();

        let def_json = json!({ "variants": variants });

        serde_json::from_value(def_json).unwrap()
    }
}

static TYPEMAP: Lazy<Mutex<HashMap<usize, RPortableType>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn resolve_type(abi: &Abi, abi_id: usize) -> Type<PortableForm> {
    let handle = TYPEMAP.lock().unwrap();

    match handle.get(&abi_id) {
        Some(r) => r.ty.clone(),
        None => {
            drop(handle);

            let ty = abi.types.get(abi_id as usize - 1).unwrap();
            let rtype = match ty {
                crate::abi::substrate::Type::Builtin { def } => {
                    let type_def = def.serde_cast();

                    let t_json = json!({
                        "def": type_def,
                    });

                    let t: Type<PortableForm> = serde_json::from_value(t_json).unwrap();

                    RPortableType { id: abi_id, ty: t }
                }
                crate::abi::substrate::Type::BuiltinArray { def } => {
                    let typedef = def.serde_cast();

                    let t_json = json!({
                        "def": TypeDef::Array(typedef),
                    });

                    let t: Type<PortableForm> = serde_json::from_value(t_json).unwrap();

                    RPortableType { id: abi_id, ty: t }
                }
                crate::abi::substrate::Type::BuiltinSequence { def } => {
                    let typedef = def.serde_cast();

                    let t_json = json!({
                        "def": TypeDef::Sequence(typedef),
                    });

                    let t: Type<PortableForm> = serde_json::from_value(t_json).unwrap();

                    RPortableType { id: abi_id, ty: t }
                }
                crate::abi::substrate::Type::Struct { path, def } => {
                    let typedef = def.serde_cast();

                    // let p = if path.is_empty() {
                    //     Path::<PortableForm>::default()
                    // } else {
                    //     let path_json = json!(path);
                    //     serde_json::from_value::<Path<PortableForm>>(path_json).unwrap()
                    // };

                    let t_json = json!({
                        // "path": p,
                        "def": TypeDef::Composite(typedef),
                    });

                    let t: Type<PortableForm> = serde_json::from_value(t_json).unwrap();

                    RPortableType { id: abi_id, ty: t }
                }
                crate::abi::substrate::Type::Enum { path, def } => {
                    let typedef = def.serde_cast();

                    // let path = if path.is_empty() {
                    //     let path_json = json!(path);
                    //     serde_json::from_value(path_json).unwrap()
                    // } else {
                    //     let path_json = json!([format!("Enum{abi_id}")]);
                    //     serde_json::from_value(path_json).unwrap()
                    // };

                    let t_json = json!({
                        // "path": path,
                        "def": TypeDef::Variant(typedef),
                    });

                    let t: Type<PortableForm> = serde_json::from_value(t_json).unwrap();

                    RPortableType { id: abi_id, ty: t }
                }
            };

            {
                let mut handle = TYPEMAP.lock().unwrap();
                handle.insert(abi_id, rtype.clone());
            }

            rtype.ty
        }
    }
}

pub fn abi_to_types(abi: &Abi) -> PortableRegistry {
    let mut types = vec![];
    for (idx, _) in abi.types.iter().enumerate() {
        let abi_id = idx + 1;
        let ty = resolve_type(abi, abi_id);

        let rty = RPortableType { id: abi_id, ty };

        types.push(rty);
    }

    let registry_json = json!({ "types": types });

    serde_json::from_value(registry_json).unwrap()
}

use hex::FromHex;

/// convert types by abusing serde_json
pub trait SerdeConversion {
    type Output;

    fn serde_cast(&self) -> Self::Output;
}

impl SerdeConversion for StorageStruct {
    type Output = Layout<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let mut fields = vec![];

        for s in self.fields.iter() {
            let name = &s.name;

            let key_buffer = <[u8; 32]>::from_hex(&s.layout.cell.key[2..]).unwrap(); // SKIP 0X
            let key = Key::new(key_buffer);
            let layout_key = LayoutKey::from(key);

            let ty = &s.layout.cell.ty;
            let cell_json = json!({ "ty": ty, "key": layout_key });

            let cell = serde_json::from_value::<CellLayout<PortableForm>>(cell_json).unwrap();
            let layout = Layout::Cell(cell);

            let flayout_json = json!({
                "name": Some(name.clone()),
                "layout": layout
            });

            let flayout =
                serde_json::from_value::<FieldLayout<PortableForm>>(flayout_json).unwrap();

            fields.push(flayout);
        }

        let s_json = json!({ "fields": fields });

        let s = serde_json::from_value::<StructLayout<PortableForm>>(s_json).unwrap();

        Layout::Struct(s)
    }
}

fn ty_from_cache(idx: usize) -> Type<PortableForm> {
    let handle = TYPEMAP.lock().unwrap();

    let ty = handle
        .iter()
        .find_map(|(k, v)| if *k as usize == idx { Some(v) } else { None })
        .unwrap();

    ty.ty.clone()
}

impl SerdeConversion for Vec<Type> {
    type Output = PortableRegistry;

    fn serde_cast(&self) -> Self::Output {
        let mut types = vec![];
        for (idx, _) in self.iter().enumerate() {
            let ty_idx = idx + 1;
            // let ty = resolve_type(abi, idx as i32 + 1);
            let ty: Type<PortableForm> = ty_from_cache(ty_idx);

            let ty_json = json!({
                "id": ty_idx,
                "ty": ty
            });

            let r_ty = serde_json::from_value::<Type<PortableForm>>(ty_json).unwrap();

            types.push(r_ty);
        }

        let registry_json = json!({ "types": types });

        serde_json::from_value(registry_json).unwrap()
    }
}

pub fn abi_to_spec(abi: &Abi) -> ContractSpec<PortableForm> {
    let constructors = abi_to_constructors(abi);
    let msgs = abi_to_msgs(abi);
    let evts = abi_to_evts(abi);
    let docs = Vec::<String>::new();

    let spec_json = json!({
        "constructors": constructors,
        "messages": msgs,
        "events": evts,
        "docs": docs
    });

    serde_json::from_value(spec_json).unwrap()
}

impl SerdeConversion for Constructor {
    type Output = ConstructorSpec<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let args = self.args.iter().map(|v| v.serde_cast()).collect::<Vec<_>>();

        let value = json!( {
            "label": self.name,
            "selector": self.selector,
            "payable": self.payable,
            "args": args,
            "docs": self.docs,
        });

        serde_json::from_value::<ConstructorSpec<PortableForm>>(value).unwrap()
    }
}

impl SerdeConversion for Message {
    type Output = MessageSpec<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let args = self.args.iter().map(|v| v.serde_cast()).collect::<Vec<_>>();

        let inner = if let Some(rt) = &self.return_type {
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
            "label": self.name,
            "selector": self.selector,
            "mutates": self.mutates,
            "payable": self.payable,
            "args": args,
            "returnType": ret_type,
            "docs": self.docs,
        });

        serde_json::from_value::<MessageSpec<PortableForm>>(value).unwrap()
    }
}

impl SerdeConversion for Param {
    type Output = MessageParamSpec<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let spec_json = json!({
            "type": self.ty.ty,
            "displayName": self.ty.display_name
        });

        let spec = serde_json::from_value::<TypeSpec<PortableForm>>(spec_json).unwrap();

        let value = json!({
            "label": self.name,
            "type": spec
        });

        serde_json::from_value::<MessageParamSpec<PortableForm>>(value).unwrap()
    }
}

pub fn abi_to_constructors(abi: &Abi) -> Vec<ConstructorSpec<PortableForm>> {
    let spec = &abi.spec;

    let out = spec
        .constructors
        .iter()
        .map(|v| v.serde_cast())
        .collect::<Vec<_>>();

    out
}

pub fn abi_to_msgs(abi: &Abi) -> Vec<MessageSpec<PortableForm>> {
    let spec = &abi.spec;

    let out = spec
        .messages
        .iter()
        .map(|v| v.serde_cast())
        .collect::<Vec<_>>();

    out
}

impl SerdeConversion for ParamIndexed {
    type Output = EventParamSpec<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let spec_json = json!({
            "type": self.param.ty.ty,
            "displayName": self.param.name
        });

        let spec = serde_json::from_value::<TypeSpec<PortableForm>>(spec_json).unwrap();

        let value = json!({
            "indexed": self.indexed,
            "label": self.param.name,
            "type": spec,
            "docs": Vec::<String>::new()
        });

        serde_json::from_value::<EventParamSpec<PortableForm>>(value).unwrap()
    }
}

impl SerdeConversion for Event {
    type Output = EventSpec<PortableForm>;

    fn serde_cast(&self) -> Self::Output {
        let args = self.args.iter().map(|v| v.serde_cast()).collect::<Vec<_>>();

        let value = json!({
            "label": self.name,
            "args": args,
            "docs":self.docs,
        });

        serde_json::from_value::<EventSpec<PortableForm>>(value).unwrap()
    }
}

pub fn abi_to_evts(abi: &Abi) -> Vec<EventSpec<PortableForm>> {
    let spec = &abi.spec;

    let out = spec
        .events
        .iter()
        .map(|v| v.serde_cast())
        .collect::<Vec<_>>();

    out
}
