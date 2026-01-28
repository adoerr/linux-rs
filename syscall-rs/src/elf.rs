#![allow(unused)]

use std::{fs, path::PathBuf};

use elf::{ElfBytes, endian::AnyEndian, section::SectionHeader};

use crate::Result;

pub fn build_id(binary: PathBuf) -> Result<String> {
    let file_data = fs::read(binary)?;
    let file = ElfBytes::<AnyEndian>::minimal_parse(file_data.as_slice())?;

    let shdr: SectionHeader = file
        .section_header_by_name(".note.gnu.build-id")?
        .ok_or("No .note.gnu.build-id section found")?;
    let _notes = file.section_data_as_notes(&shdr)?;

    Ok("".to_string())
}
