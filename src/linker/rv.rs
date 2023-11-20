use std::{
    ffi::CString,
    fs::File,
    io::{Read, Write},
};

use tempfile::tempdir;

pub fn link(input: &[u8], name: &str) -> Vec<u8> {
    let dir = tempdir().expect("failed to create temp directory for linking");

    let object_filename = dir.path().join(format!("{name}.o"));
    let res_filename = dir.path().join(format!("{name}.so"));
    let linker_script_filename = dir.path().join("linker.ld");

    let mut objectfile =
        File::create(object_filename.clone()).expect("failed to create object file");

    objectfile
        .write_all(input)
        .expect("failed to write object file to temp file");

    let mut linker_script =
        File::create(linker_script_filename.clone()).expect("failed to create linker script");

    linker_script
        .write_all(
            br##"
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
}"##,
        )
        .expect("failed to write linker script to temp file");

    let command_line = vec![
        CString::new("--error-limit=0").unwrap(),
        CString::new("--relocatable").unwrap(),
        CString::new("--emit-relocs").unwrap(),
        CString::new("--no-relax").unwrap(),
        CString::new(
            linker_script_filename
                .to_str()
                .expect("temp path should be unicode"),
        )
        .unwrap(),
        CString::new(
            object_filename
                .to_str()
                .expect("temp path should be unicode"),
        )
        .unwrap(),
        CString::new("-o").unwrap(),
        CString::new(res_filename.to_str().expect("temp path should be unicode")).unwrap(),
    ];

    assert!(!super::elf_linker(&command_line), "linker failed");

    let mut output = Vec::new();
    // read the whole file
    let mut outputfile = File::open(res_filename).expect("output file should exist");

    outputfile
        .read_to_end(&mut output)
        .expect("failed to read output file");

    output
}
