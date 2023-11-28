use std::ffi::CString;

use itertools::Itertools;
use tempfile::tempdir;

pub fn link(input: &[u8], name: &str) -> Vec<u8> {
    let dir = tempdir().expect("failed to create temp directory for linking");

    let object_filename = dir.path().join(name).with_extension("o");
    let res_filename = dir.path().join(name).with_extension("so");
    let linker_script_filename = dir.path().join("linker.ld");

    std::fs::write(&object_filename, input).expect("failed to write object file to temp file");

    let linker_script = br##"
    SECTIONS {
        . = 0x10000;
        .rodata : { *(.rodata) *(.rodata.*) }
        .data.rel.ro : { *(.data.rel.ro) *(.data.rel.ro.*) }
        .got : { *(.got) *(.got.*) }
    
        . = ALIGN(0x4000);
        .data : { *(.sdata) *(.data) }
        .bss : { *(.sbss) *(.bss) *(.bss.*) }
    
        . = 0xf0000000;
    
        .text : { KEEP(*(.text.polkavm_export)) *(.text .text.*) }
    
        /DISCARD/ : { *(.eh_frame) }
        . = ALIGN(4);
    }"##;
    std::fs::write(&linker_script_filename, linker_script)
        .expect("failed to write linker script to temp file");

    let ld_args = [
        "--error-limit=0",
        "--relocatable",
        "--emit-relocs",
        "--no-relax",
        "--gc-sections",
        "--library-path",
        "/opt/clang-rv32e/lib/linux",
        "--library",
        "clang_rt.builtins-riscv32",
        linker_script_filename.to_str().expect("should be unicode"),
        object_filename.to_str().expect("should be unicode"),
        "-o",
        res_filename.to_str().expect("should be unicode"),
    ]
    .iter()
    .map(|arg| CString::new(*arg).unwrap())
    .collect_vec();

    assert!(!super::elf_linker(&ld_args), "linker failed");

    let mut config = polkavm_linker::Config::default();
    config.set_strip(true);
    let code = std::fs::read(&res_filename).unwrap();
    //std::fs::write("/home/cyrill/mess/solang/out.so", &code).unwrap();
    let output = match polkavm_linker::program_from_elf(config, &code) {
        Ok(blob) => blob.as_bytes().to_vec(),
        Err(reason) => panic!("polkavm linker failed: {}", reason),
    };

    polkavm_common::program::ProgramBlob::parse(&output[..]).expect("Valid PVM blob after linker");

    output
}
