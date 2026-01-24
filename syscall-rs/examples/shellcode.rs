//!
//! This file is part of syscall-rs
//!

use std::{mem, ptr};

use syscall::{syscall, Result};

fn main() -> Result<()> {
    #[cfg(target_arch = "x86_64")]
    let code: &[u8] = &[
        0x90, // NOP
        0x90, // NOP
        0x90, // NOP
        0xc3, // RET
    ];

    #[cfg(target_arch = "aarch64")]
    let code: &[u8] = &[
        0x1f, 0x20, 0x03, 0xd5, // NOP
        0x1f, 0x20, 0x03, 0xd5, // NOP
        0x1f, 0x20, 0x03, 0xd5, // NOP
        0xc0, 0x03, 0x5f, 0xd6, // RET
    ];

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    compile_error!("Unsupported architecture");

    env_logger::init();

    // 1. get the system page size (usually 4096 bytes)
    let page_size = syscall!(sysconf(libc::_SC_PAGE_SIZE))? as usize;

    let ptr = unsafe {
        // 2. allocate memory using mmap
        libc::mmap(
            ptr::null_mut(),
            page_size,
            // memory has READ, WRITE and EXEC permissions
            libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
            // memory is not backed by any file and private to the current process
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        )
    };

    if ptr == libc::MAP_FAILED {
        panic!("mmap failed");
    }

    log::info!("Memory allocated at: {:p}", ptr);

    unsafe {
        // 3. copy shellcode into the allocated memory
        ptr::copy_nonoverlapping(code.as_ptr(), ptr as *mut u8, code.len());
    }

    log::info!("Shellcode copied to allocated memory.");

    // 4. change memory protection to READ | EXEC
    let res = unsafe { libc::mprotect(ptr, page_size, libc::PROT_READ | libc::PROT_EXEC) };

    if res != 0 {
        unsafe {
            libc::munmap(ptr, page_size);
        }
        panic!("mprotect failed");
    }

    log::info!("Memory protection changed to READ | EXEC. Executing shellcode...");

    // 5. cast memory pointer to a function
    let func: extern "C" fn() = unsafe { mem::transmute(ptr) };

    // 6. call the shellcode function
    func();

    log::info!("Controlled execution returned here.");

    unsafe {
        // 7. clean up: unmap the allocated memory
        libc::munmap(ptr, page_size);
    }

    Ok(())
}
