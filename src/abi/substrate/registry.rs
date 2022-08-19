use crate::ast;
use ink_metadata::{
    layout::{self as inklayout},
    ContractSpec,
};

pub mod converter;
use converter::SerdeConversion;

use scale_info::{form::PortableForm, PortableRegistry};

use super::gen_abi;

pub fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

type Artifact = (
    PortableRegistry,
    inklayout::Layout<PortableForm>,
    ContractSpec<PortableForm>,
);

pub fn gen_project(contract_no: usize, ns: &ast::Namespace) -> anyhow::Result<Artifact> {
    let a = gen_abi(contract_no, ns);

    let layout = a.storage.structs.serde_cast();
    let registry = converter::abi_to_types(&a);
    let spec = converter::abi_to_spec(&a);

    Ok((registry, layout, spec))
}
