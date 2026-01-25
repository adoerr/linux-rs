use std::{
    ffi::CString,
    fs::File,
    io::{Read, Write},
    os::unix::io::{FromRawFd, IntoRawFd},
};

use nix::{
    sys::{
        signal::Signal,
        wait::{WaitStatus, waitpid},
    },
    unistd::{ForkResult, execv, fork},
};
use syscall::{Result, syscall};

fn main() -> Result<()> {
    env_logger::init();

    // define the target binary (we use /bin/ls as a base)
    let target_path = "/bin/ls";

    // fork the process
    match unsafe { fork()? } {
        ForkResult::Child => {
            // read target binary
            let mut binary = File::open(target_path)?;
            let mut binary_buffer = Vec::new();
            binary.read_to_end(&mut binary_buffer)?;

            // patch entry point
            let entry_offset = entry_offset(&binary_buffer).unwrap();

            log::info!("target binary entry point offset: {:#x}", entry_offset);

            // dummy payload:
            #[cfg(target_arch = "x86_64")]
            // x86_64: 0xCC is the opcode for INT 3 (Breakpoint).
            let payload: &[u8] = &[0xCC];

            #[cfg(target_arch = "aarch64")]
            // aarch64: 0xd4200000 is brk #0 (Little Endian: 00 00 20 d4)
            let payload: &[u8] = &[0x00, 0x00, 0x20, 0xd4];

            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            compile_error!("Unsupported architecture");

            // check that payload fits in the target binary
            if entry_offset as usize + payload.len() > binary_buffer.len() {
                panic!("payload size too large");
            }

            // overwrite existing code at entry point
            for (i, b) in payload.iter().enumerate() {
                binary_buffer[entry_offset as usize + i] = *b;
            }

            log::info!("patched binary in memory buffer");

            let name = CString::new("hollow_ls")?;

            let fd = syscall!(memfd_create(name.as_ptr(), libc::MFD_CLOEXEC))?;
            // write modified binary
            let mut mem_file = unsafe { File::from_raw_fd(fd) };
            mem_file.write_all(&binary_buffer)?;
            // convert back to raw fd (prevent close)
            let fd = mem_file.into_raw_fd();

            // exec via /proc/self/fd/N
            let fd_path = CString::new(format!("/proc/self/fd/{}", fd))?;

            // reuse the target name for argv[0].
            let arg0 = CString::new("hollow_ls")?;
            let args = &[arg0.as_c_str()];

            log::info!("executing memfd via {}", fd_path.to_string_lossy());

            execv(&fd_path, args)?;
        }
        ForkResult::Parent { child } => {
            log::info!("spawned child PID: {}", child);

            match waitpid(child, None)? {
                WaitStatus::Signaled(_, Signal::SIGTRAP, _) => {
                    log::info!("child hit our injected trap (killed by SIGTRAP)!");
                    log::info!("execution flow was successfully hijacked.");
                }
                status => {
                    log::info!("unexpected status: {:?}", status);
                }
            }
        }
    }

    Ok(())
}

// Parses the ELF header to find the file offset of the entry point.
//
// ELF-64 Header Layout (Offset 0x00 - 0x40):
//
//   0x00                                     0x10
//   +---------------------------------------+
//   |  0x7f 'E' 'L' 'F' | Class | Data | ...|  e_ident (16 bytes)
//   +-------------------+-------+-----------+
//   |      e_type       |     e_machine     |  0x10: Type & Machine
//   +-------------------+-------------------+
//   |      e_version    |                   |  0x14: Version
//   +-------------------+-------------------+
//   |                                       |
//   |                e_entry                |  0x18: Entry point address (64-bit)
//   |                                       |
//   +---------------------------------------+
//   |                                       |
//   |                e_phoff                |  0x20: Program Header Offset (64-bit)
//   |                                       |
//   +---------------------------------------+
//   |                                       |
//   |                e_shoff                |  0x28: Section Header Offset (64-bit)
//   |                                       |
//   +---------------------------------------+
//   |      e_flags      |     e_ehsize      |  0x30: Flags & Header Size
//   +-------------------+-------------------+
//   |     e_phentsize   |     e_phnum       |  0x36: Prog Header Entry Size & Count
//   +-------------------+-------------------+
//   |     e_shentsize   |     e_shnum       |  0x3A: Sect Header Entry Size & Count
//   +-------------------+-------------------+
//   |     e_shstrndx    |        ...        |  0x3E: String Table Index
//   +-------------------+-------------------+
//
//   Key fields used in this parser:
//
//   - 0x00 (4 bytes): Magic Number (\x7fELF)
//   - 0x18 (8 bytes): e_entry (Virtual Address)
//   - 0x20 (8 bytes): e_phoff (File Offset to Program Headers)
//   - 0x36 (2 bytes): e_phentsize (Size of one PH entry)
//   - 0x38 (2 bytes): e_phnum (Number of PH entries)
//
fn entry_offset(binary: &[u8]) -> Option<u64> {
    // check for ELF magic number
    if binary.len() < 64 || &binary[0..4] != b"\x7fELF" {
        return None;
    }

    // helper closures for value extraction at given positions
    let u16_at = |pos: usize| u16::from_ne_bytes(binary[pos..pos + 2].try_into().unwrap());
    let u32_at = |pos: usize| u32::from_ne_bytes(binary[pos..pos + 4].try_into().unwrap());
    let u64_at = |pos: usize| u64::from_ne_bytes(binary[pos..pos + 8].try_into().unwrap());

    // require EI_CLASS to be 2 (64-bit)
    if binary[4] != 2 {
        return None;
    }

    // virtual address of entry point
    let e_entry = u64_at(0x18);
    // program header table offset
    let e_phoff = u64_at(0x20);
    // number of program header entries
    let e_phnum = u16_at(0x38);
    // size of each program header entry
    let p_ent_size = u16_at(0x36) as usize;

    // start at program header table offset
    let mut offset = e_phoff as usize;
    // size of each program header entry
    const PROG_HEADER_SIZE: usize = 56;

    //   ELF-64 Program Header Entry (56 bytes total)
    //   Describes a segment of the file to be mapped into memory.//
    //
    //   Offset   Field         Size      Description
    //   +------+-------------+---------+----------------------------------------------+
    //   | 0x00 | p_type      | 4 bytes | Segment Type (1 = PT_LOAD)                   |
    //   | 0x04 | p_flags     | 4 bytes | Segment Flags (Read/Write/Exec permissions)  |
    //   +------+-------------+---------+----------------------------------------------+
    //   | 0x08 | p_offset    | 8 bytes | Offset of segment data in the file           |
    //   +------+-------------+---------+----------------------------------------------+
    //   | 0x10 | p_vaddr     | 8 bytes | Virtual address where segment begins in RAM  |
    //   +------+-------------+---------+----------------------------------------------+
    //   | 0x18 | p_paddr     | 8 bytes | Physical address (unused on standard Linux)  |
    //   +------+-------------+---------+----------------------------------------------+
    //   | 0x20 | p_filesz    | 8 bytes | Size of segment in the file                  |
    //   +------+-------------+---------+----------------------------------------------+
    //   | 0x28 | p_memsz     | 8 bytes | Size of segment in memory (can be > filesz)  |
    //   +------+-------------+---------+----------------------------------------------+
    //   | 0x30 | p_align     | 8 bytes | Alignment of segment in memory               |
    //   +------+-------------+---------+----------------------------------------------+
    //
    //   Load Calculation Logic:
    //
    //   If `e_entry` (Global Entry Point) is >= `p_vaddr` and < `p_vaddr` + `p_memsz`:
    //     Physical Entry Offset = e_entry - p_vaddr + p_offset

    // for each program header entry
    for _ in 0..e_phnum {
        // ensure we don't read beyond buffer
        if offset + PROG_HEADER_SIZE > binary.len() {
            break;
        }

        // read the segment type (p_type) from the program header.
        let p_type = u32_at(offset);

        // we are interested in PT_LOAD loadable segments (type 1).
        if p_type == 1 {
            //
            let p_offset = u64_at(offset + 8);
            let p_vaddr = u64_at(offset + 16);
            let p_memsz = u64_at(offset + 40);

            // check if entry point lies within this segment's memory range
            if e_entry >= p_vaddr && e_entry < p_vaddr + p_memsz {
                // file offset corresponding to entry point
                return Some(e_entry - p_vaddr + p_offset);
            }
        }

        // next program header entry.
        offset += p_ent_size;
    }

    // no matching segment found
    None
}
