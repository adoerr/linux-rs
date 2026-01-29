#![allow(unused)]

use std::path::Path;

use elf::{ElfBytes, endian::AnyEndian};

use crate::Result;

/// Extract the Build ID from an ELF binary.
///
/// This function parses the ELF headers of the given binary path and searches for
/// the `.note.gnu.build-id` section. If found, it extracts the Build ID as a
/// hexadecimal string.
pub fn build_id(binary: &Path) -> Result<Option<String>> {
    let data = std::fs::read(binary)?;
    let file = ElfBytes::<AnyEndian>::minimal_parse(&data)?;

    // get the section headers and the section header string table
    let (headers, strtab) = file.section_headers_with_strtab()?;

    let headers = match headers {
        Some(h) => h,
        None => return Ok(None),
    };

    let strtab = match strtab {
        Some(s) => s,
        None => return Ok(None),
    };

    let endian = file.ehdr.endianness;

    // iterate over all section headers
    for shdr in headers.iter() {
        // check if the section name is ".note.gnu.build-id"
        if let Ok(".note.gnu.build-id") = strtab.get(shdr.sh_name as usize) {
            // get raw section data
            let note_data = file.section_data(&shdr)?.0;

            let is_le = matches!(endian, AnyEndian::Little);

            // helper to read u32 based on endianness
            let read_u32 = |b: &[u8]| {
                let bytes: [u8; 4] = b.try_into().unwrap();
                if is_le {
                    u32::from_le_bytes(bytes)
                } else {
                    u32::from_be_bytes(bytes)
                }
            };

            let mut pos = 0;

            // iterate over section notes
            while pos + 12 <= note_data.len() {
                // read note header: namesz, descsz, type
                let n_namesz = read_u32(&note_data[pos..pos + 4]) as usize;
                let n_descsz = read_u32(&note_data[pos + 4..pos + 8]) as usize;
                let n_type = read_u32(&note_data[pos + 8..pos + 12]);
                pos += 12;

                // create 4-byte aligned sizes
                let name_align = (n_namesz + 3) & !3;
                let desc_align = (n_descsz + 3) & !3;

                if pos + name_align + desc_align > note_data.len() {
                    break;
                }

                // check for NT_GNU_BUILD_ID (type 3) and name "GNU\0"
                if n_type == 3 && n_namesz == 4 && &note_data[pos..pos + 4] == b"GNU\0" {
                    let desc = &note_data[pos + name_align..pos + name_align + n_descsz];
                    // convert build ID bytes to hex string
                    let hex: String = desc.iter().map(|b| format!("{:02x}", b)).collect();
                    return Ok(Some(hex));
                }

                pos += name_align + desc_align;
            }
        }
    }

    Ok(None)
}
