use std::{cell::RefCell, collections::HashMap, sync::Mutex};

use crate::{ast, sema::tags::render};
use hex::FromHex;
use ink_metadata::{
    layout::{self as inklayout, CellLayout, FieldLayout, LayoutKey, StructLayout},
    ConstructorSpec, ContractSpec, EventParamSpec, EventSpec, InkProject, MessageParamSpec,
    MessageSpec, ReturnTypeSpec, TypeSpec,
};

use once_cell::sync::{Lazy, OnceCell};
use scale_info::{
    build::{
        field_state::{TypeAssigned, TypeNotAssigned},
        FieldBuilder, Fields, Variants,
    },
    meta_type, IntoPortable, MetaType, Path, Registry, Type as ScaleType, TypeDefArray,
    TypeDefPrimitive, TypeDefSequence, TypeInfo,
};
use solang_parser::pt;

use super::{ty_to_abi, Abi, ParamType, PrimitiveDef, Spec, Storage, StorageStruct};

pub fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

mod command {
    use std::marker::PhantomData;

    pub struct GenMeta {}

    pub struct GenTypeSpec<T>(PhantomData<T>);

    pub struct GenCellLayout;

    pub struct DecFieldBuilder<T>(PhantomData<T>);
}

trait CommandOutput {
    type Input;
    type Output;

    fn generate<T: TypeInfo + 'static>(input: Self::Input, _: T) -> Self::Output;

    fn wrap_and_gen(ty: ScaleType, input: Self::Input) -> Self::Output {
        static STATIC_TYLAYOUT: OnceCell<ScaleType> = OnceCell::new();
        let _ = STATIC_TYLAYOUT.set(ty);

        struct Inplace {}

        impl TypeInfo for Inplace {
            type Identity = Self;

            fn type_info() -> ScaleType {
                STATIC_TYLAYOUT.get().map(|v| v.clone()).unwrap()
            }
        }

        Self::generate::<Inplace>(input, Inplace {})
    }
}

impl CommandOutput for command::GenMeta {
    type Input = ();

    type Output = MetaType;

    fn generate<T: TypeInfo + 'static>(_: Self::Input, _: T) -> Self::Output {
        meta_type::<T>()
    }
}

impl CommandOutput for command::GenCellLayout {
    type Input = LayoutKey;

    type Output = CellLayout;

    fn generate<T: TypeInfo + 'static>(input: Self::Input, _: T) -> Self::Output {
        CellLayout::new::<T>(input)
    }
}

impl<I> CommandOutput for command::GenTypeSpec<I>
where
    I: IntoIterator<Item = &'static str>,
{
    type Input = I;

    type Output = TypeSpec;

    fn generate<T: TypeInfo + 'static>(input: Self::Input, _: T) -> Self::Output {
        TypeSpec::with_name_segs::<T, I>(input)
    }
}

impl<I> CommandOutput for command::DecFieldBuilder<I> {
    type Input = FieldBuilder<I, TypeNotAssigned>;

    type Output = FieldBuilder<I, TypeAssigned>;

    fn generate<T: TypeInfo + 'static>(input: Self::Input, _: T) -> Self::Output {
        input.ty::<T>()
    }
}

static TYPEMAP: Lazy<Mutex<HashMap<usize, ScaleType>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn abi_to_type(abi_id: usize, abi: &Abi, registry: &mut Registry) -> ScaleType {
    let handle = TYPEMAP.lock().expect("unable to lock");
    let abi_handle = RefCell::new(abi);
    let registry_handle = RefCell::new(registry);

    let ty = match handle.get(&abi_id) {
        Some(ty) => ty.clone(),
        None => {
            drop(handle);

            let abi_type = {
                abi_handle
                    .borrow()
                    .types
                    .get(abi_id - 1) // ABI index starts from 1
                    .expect("unable to find type in ABI")
            };

            match abi_type {
                super::Type::Builtin { def } => {
                    let mut registry = registry_handle.borrow_mut();
                    let ty = primitive_to_type(abi_id, def, &mut registry);

                    ty.clone()
                }
                super::Type::BuiltinArray { def } => {
                    let abi = abi_handle.borrow();
                    let mut registry = registry_handle.borrow_mut();
                    // get def of child element
                    let e = abi_to_type(def.array.ty, &abi, &mut registry);
                    let e_meta = command::GenMeta::wrap_and_gen(e, ());

                    let ty = ScaleType::from(TypeDefArray::new(def.array.len as u32, e_meta));

                    let meta = command::GenMeta::wrap_and_gen(ty.clone(), ());
                    registry.register_type(&meta);

                    {
                        let mut handle = TYPEMAP.lock().expect("unable to lock");
                        handle.insert(abi_id, ty.clone());
                    }

                    ty
                }
                super::Type::BuiltinSequence { def } => {
                    let abi = abi_handle.borrow();
                    let mut registry = registry_handle.borrow_mut();
                    let e = abi_to_type(def.sequence.ty, &abi, &mut registry);

                    let e_meta = command::GenMeta::wrap_and_gen(e, ());
                    let ty = ScaleType::from(TypeDefSequence::new(e_meta));

                    let meta = command::GenMeta::wrap_and_gen(ty.clone(), ());
                    registry.register_type(&meta);

                    {
                        let mut handle = TYPEMAP.lock().expect("unable to lock");
                        handle.insert(abi_id, ty.clone());
                    }

                    ty
                }
                super::Type::Struct { path, def } => {
                    let segments = path
                        .iter()
                        .map(|e| string_to_static_str(e.clone()))
                        .collect::<Vec<_>>();
                    let path = Path::from_segments(segments).expect("unable to construct path");
                    let named = def.composite.fields.iter().all(|f| f.name.is_some());

                    let composite = if named {
                        let mut fields_builder = Fields::named();

                        for f in &def.composite.fields {
                            let f_ty = f.ty.clone();
                            fields_builder = fields_builder.field(|b| {
                                let abi = abi_handle.borrow();
                                let mut registry = registry_handle.borrow_mut();
                                let e = abi_to_type(f_ty, &abi, &mut registry);

                                command::DecFieldBuilder::wrap_and_gen(
                                    e,
                                    b.name(
                                        f.name
                                            .as_ref()
                                            .map(|n| string_to_static_str(n.clone()))
                                            .expect("should be named"),
                                    ),
                                )
                            });
                        }
                        ScaleType::builder().path(path).composite(fields_builder)
                    } else {
                        let mut fields_builder = Fields::unnamed();

                        for f in &def.composite.fields {
                            fields_builder = fields_builder.field(|b| {
                                let abi = abi_handle.borrow_mut();
                                let mut registry = registry_handle.borrow_mut();
                                let e = abi_to_type(f.ty, &abi, &mut registry);

                                command::DecFieldBuilder::wrap_and_gen(e, b)
                            });
                        }
                        ScaleType::builder().path(path).composite(fields_builder)
                    };

                    let meta = command::GenMeta::wrap_and_gen(composite.clone(), ());
                    let mut registry = registry_handle.borrow_mut();
                    registry.register_type(&meta);

                    {
                        let mut handle = TYPEMAP.lock().expect("unable to lock");
                        handle.insert(abi_id, composite.clone());
                    }

                    composite
                }
                super::Type::Enum { path, def } => {
                    let p = path
                        .iter()
                        .map(|e| string_to_static_str(e.clone()))
                        .collect::<Vec<_>>();
                    let path = Path::from_segments(p).expect("unable to construct path");
                    let mut variants = Variants::new();
                    for v in &def.variant.variants {
                        variants = variants.variant_unit(
                            string_to_static_str(v.name.clone()),
                            v.discriminant as u8,
                        )
                    }

                    let ty = ScaleType::builder().path(path).variant(variants);

                    let meta = command::GenMeta::wrap_and_gen(ty.clone(), ());
                    let mut registry = registry_handle.borrow_mut();
                    registry.register_type(&meta);

                    {
                        let mut handle = TYPEMAP.lock().expect("unable to lock");
                        handle.insert(abi_id, ty.clone());
                    }

                    ty
                }
            }
        }
    };

    ty
}

fn primitive_to_type(abi_id: usize, def: &PrimitiveDef, registry: &mut Registry) -> ScaleType {
    let ty = match def.primitive.as_str() {
        "bool" => ScaleType::from(TypeDefPrimitive::Bool),
        "str" => ScaleType::from(TypeDefPrimitive::Str),
        "u8" => ScaleType::from(TypeDefPrimitive::U8),
        "u16" => ScaleType::from(TypeDefPrimitive::U16),
        "u32" => ScaleType::from(TypeDefPrimitive::U32),
        "u64" => ScaleType::from(TypeDefPrimitive::U64),
        "u128" => ScaleType::from(TypeDefPrimitive::U128),
        "u256" => ScaleType::from(TypeDefPrimitive::U256),
        "i8" => ScaleType::from(TypeDefPrimitive::I8),
        "i16" => ScaleType::from(TypeDefPrimitive::I16),
        "i32" => ScaleType::from(TypeDefPrimitive::I32),
        "i64" => ScaleType::from(TypeDefPrimitive::I64),
        "i128" => ScaleType::from(TypeDefPrimitive::I128),
        "AccountId" => {
            let e = ScaleType::from(TypeDefPrimitive::U8);
            let e_meta = command::GenMeta::wrap_and_gen(e, ());

            ScaleType::from(TypeDefArray::new(32, e_meta))
        }
        _ => {
            unimplemented!("not supported types")
        }
    };

    {
        let mut handle = TYPEMAP.lock().expect("unable to lock");
        handle.insert(abi_id, ty.clone());
    }

    let meta = command::GenMeta::wrap_and_gen(ty.clone(), ());

    registry.register_type(&meta);

    ty
}

fn build_segments(ty: &ScaleType, param: &ParamType) -> Vec<&'static str> {
    let out = if ty.path().segments().is_empty() {
        param
            .display_name
            .iter()
            .map(|v| string_to_static_str(v.clone()))
            .collect()
    } else {
        ty.path()
            .segments()
            .iter()
            .map(|v| string_to_static_str(v.to_string()))
            .collect()
    };

    out
}

pub fn gen_project(contract_no: usize, ns: &ast::Namespace) -> anyhow::Result<InkProject> {
    let registry = Registry::new();

    let abi = Abi {
        types: Vec::new(),
        storage: Storage {
            structs: StorageStruct { fields: Vec::new() },
        },
        spec: Spec {
            constructors: Vec::new(),
            messages: Vec::new(),
            events: Vec::new(),
        },
    };

    let abi_handle = RefCell::new(abi);
    let registry_handle = RefCell::new(registry);

    let fields = ns.contracts[contract_no]
        .layout
        .iter()
        .filter_map(|layout| {
            let var = &ns.contracts[layout.contract_no].variables[layout.var_no];

            // mappings and large types cannot be represented
            if !var.ty.contains_mapping(ns) && var.ty.fits_in_memory(ns) {
                let mut abi = abi_handle.borrow_mut();

                let mut registry = registry_handle.borrow_mut();
                let inner: FieldLayout;

                let key = <[u8; 32]>::from_hex(format!("{:064X}", layout.slot))
                    .expect("failed to parse hex string");

                let layout_key = LayoutKey::from(key);
                let param = ty_to_abi(&layout.ty, ns, &mut abi);
                let ty = abi_to_type(param.ty, &abi, &mut registry);

                let flayout = command::GenCellLayout::wrap_and_gen(ty.clone(), layout_key);

                inner = FieldLayout::new(Some(string_to_static_str(var.name.clone())), flayout);

                Some(inner)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let slayout = StructLayout::new(fields);
    let layout = inklayout::Layout::Struct(slayout);

    {
        let r = registry_handle.borrow();
    }

    // TODO: generate constructors
    let mut constructors: Vec<ConstructorSpec> = ns.contracts[contract_no]
        .functions
        .iter()
        .filter_map(|function_no| {
            let f = &ns.functions[*function_no];
            if f.is_constructor() {
                let payable = matches!(f.mutability, ast::Mutability::Payable(_));
                let mut abi = abi_handle.borrow_mut();
                let mut registry = registry_handle.borrow_mut();

                let selector = f.selector().to_be_bytes();

                let args = f
                    .params
                    .iter()
                    .map(|p| {
                        // still get or create the types to ABI registry
                        let e = ty_to_abi(&p.ty, ns, &mut abi);

                        // generate TypeDef to be put into registry
                        let ty = abi_to_type(e.ty, &abi, &mut registry);

                        let segments = build_segments(&ty, &e);

                        let spec = command::GenTypeSpec::wrap_and_gen(ty, segments);

                        // generate Inplace struct that use the TypeDef as it's TypeInfo output
                        let label = string_to_static_str(p.name_as_str().to_string());

                        let arg = MessageParamSpec::new(label).of_type(spec).done();

                        arg
                    })
                    .collect::<Vec<_>>();

                let inner = ConstructorSpec::from_label("new")
                    .selector(selector)
                    .args(args)
                    .docs(vec![string_to_static_str(render(&f.tags))])
                    .payable(payable)
                    .done();

                Some(inner)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // insert default constructor
    if let Some((f, _)) = &ns.contracts[contract_no].default_constructor {
        let payable = matches!(f.mutability, ast::Mutability::Payable(_));

        let selector = f.selector().to_be_bytes();

        let args = f
            .params
            .iter()
            .map(|p| {
                let mut abi = abi_handle.borrow_mut();
                let mut registry = registry_handle.borrow_mut();
                // still get or create the types to ABI registry
                let e = ty_to_abi(&p.ty, ns, &mut abi);

                // generate TypeDef to be put into registry
                let ty = abi_to_type(e.ty, &abi, &mut registry);

                let segments = build_segments(&ty, &e);

                let spec = command::GenTypeSpec::wrap_and_gen(ty, segments);

                // generate Inplace struct that use the TypeDef as it's TypeInfo output
                let label = string_to_static_str(p.name_as_str().to_string());

                let arg = MessageParamSpec::new(label).of_type(spec).done();

                arg
            })
            .collect::<Vec<_>>();

        let inner = ConstructorSpec::from_label("new")
            .selector(selector)
            .args(args)
            .docs(vec![string_to_static_str(render(&f.tags))])
            .payable(payable)
            .done();

        constructors.push(inner);
    }

    // TODO: generate messages
    let messages = ns.contracts[contract_no]
        .all_functions
        .keys()
        .filter_map(|function_no| {
            let func = &ns.functions[*function_no];

            if let Some(base_contract_no) = func.contract_no {
                if ns.contracts[base_contract_no].is_library() {
                    return None;
                }
            }

            Some(func)
        })
        .filter(|f| match f.visibility {
            pt::Visibility::Public(_) | pt::Visibility::External(_) => {
                f.ty == pt::FunctionTy::Function
            }
            _ => false,
        })
        .map(|f| {
            let mut abi = abi_handle.borrow_mut();
            let mut registry = registry_handle.borrow_mut();
            let payable = matches!(f.mutability, ast::Mutability::Payable(_));
            let selector = f.selector().to_be_bytes();

            let args = f
                .params
                .iter()
                .map(|p| {
                    // still get or create the types to ABI registry
                    let e = ty_to_abi(&p.ty, ns, &mut abi);

                    // generate TypeDef to be put into registry
                    let ty = abi_to_type(e.ty, &abi, &mut registry);

                    let label = string_to_static_str(p.name_as_str().to_string());
                    let mut segments = build_segments(&ty, &e);
                    if segments.is_empty() {
                        segments = vec![label];
                    }

                    let spec = command::GenTypeSpec::wrap_and_gen(ty, segments);

                    MessageParamSpec::new(label).of_type(spec).done()
                })
                .collect::<Vec<_>>();

            let mutates = matches!(
                f.mutability,
                ast::Mutability::Payable(_) | ast::Mutability::Nonpayable(_)
            );

            let contract = &ns.contracts[contract_no];
            let contract_name = &contract.name;
            let function_name = &f.name;

            let segments = [contract_name, function_name, &String::from("return_type")]
                .into_iter()
                .map(|v| string_to_static_str(v.clone()))
                .collect::<Vec<_>>();

            let return_spec = match f.returns.len() {
                0 => ReturnTypeSpec::new(None),
                1 => {
                    let e = ty_to_abi(&f.returns[0].ty, ns, &mut abi);

                    // generate TypeDef to be put into registry
                    let ty = abi_to_type(e.ty, &abi, &mut registry);

                    let spec = command::GenTypeSpec::wrap_and_gen(ty, segments);

                    // TODO: create Inplace struct
                    ReturnTypeSpec::new(spec)
                }
                _ => {
                    let path = Path::from_segments(segments).expect("invalid path");
                    let is_named = f.returns.iter().all(|f| f.name.is_some());

                    let ty = if is_named {
                        let mut fields_builder = Fields::named();

                        for field in f.returns.iter() {
                            fields_builder = fields_builder.field(|b| {
                                let mut abi = abi_handle.borrow_mut();
                                let mut registry = registry_handle.borrow_mut();
                                let name = &field
                                    .name
                                    .iter()
                                    .last()
                                    .map(|id| string_to_static_str(id.name.clone()))
                                    .unwrap();

                                let e = ty_to_abi(&field.ty, ns, &mut abi);

                                // generate TypeDef to be put into registry
                                let ty = abi_to_type(e.ty, &abi, &mut registry);

                                command::DecFieldBuilder::wrap_and_gen(ty, b.name(name))
                            });
                        }

                        // TODO: register type in registry;
                        let ret_type = scale_info::Type::builder()
                            .docs(&[])
                            .path(path)
                            .composite(fields_builder);

                        ret_type
                    } else {
                        let mut fields_builder = Fields::unnamed();

                        for field in f.returns.iter() {
                            fields_builder = fields_builder.field(|b| {
                                let mut abi = abi_handle.borrow_mut();
                                let mut registry = registry_handle.borrow_mut();
                                let e = ty_to_abi(&field.ty, ns, &mut abi);

                                // generate TypeDef to be put into registry
                                let ty = abi_to_type(e.ty, &mut abi, &mut registry);

                                command::DecFieldBuilder::wrap_and_gen(ty, b)
                            });
                        }

                        let ret_type = scale_info::Type::builder()
                            .docs(&[])
                            .path(path)
                            .composite(fields_builder);

                        let meta = command::GenMeta::wrap_and_gen(ret_type.clone(), ());
                        registry.register_type(&meta);

                        ret_type
                    };

                    let segments = ty
                        .path()
                        .segments()
                        .iter()
                        .map(|v| string_to_static_str(v.to_string()))
                        .collect::<Vec<_>>();

                    let spec = command::GenTypeSpec::wrap_and_gen(ty, segments);

                    let rt = ReturnTypeSpec::new(spec);

                    rt
                }
            };

            let out = MessageSpec::from_label(string_to_static_str(f.name.to_string()))
                .selector(selector)
                .args(args)
                .docs(vec![string_to_static_str(render(&f.tags))])
                .mutates(mutates)
                .returns(return_spec)
                .payable(payable)
                .done();

            out
        })
        .collect::<Vec<_>>();

    let events = ns.contracts[contract_no]
        .sends_events
        .iter()
        .map(|event_no| {
            let event = &ns.events[*event_no];

            let e_name = event.name.to_owned();

            let args = event
                .fields
                .iter()
                .map(|p| {
                    let mut abi = abi_handle.borrow_mut();
                    let mut registry = registry_handle.borrow_mut();
                    let e = ty_to_abi(&p.ty, ns, &mut abi);

                    // generate TypeDef to be put into registry
                    let ty = abi_to_type(e.ty, &abi, &mut registry);

                    let segments = e
                        .display_name
                        .iter()
                        .map(|v| string_to_static_str(v.to_string()))
                        .collect::<Vec<_>>();

                    let spec = command::GenTypeSpec::wrap_and_gen(ty, segments);

                    let arg = EventParamSpec::new(string_to_static_str(p.name_as_str().to_owned()))
                        .indexed(p.indexed)
                        .of_type(spec)
                        .done();

                    arg
                })
                .collect::<Vec<_>>();
            let docs = vec![string_to_static_str(render(&event.tags))];

            EventSpec::new(string_to_static_str(e_name))
                .args(args)
                .docs(docs)
                .done()
        })
        .collect::<Vec<_>>();

    let spec = ContractSpec::new()
        .constructors(constructors)
        .messages(messages)
        .events(events)
        .done();

    let project = InkProject::new(layout, spec);

    Ok(project)
}
