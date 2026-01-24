//!
//! This file is part of syscall-rs
//!

use std::{mem, num::NonZeroUsize, ptr};

use syscall::{MapFlags, ProtFlags, Result, mmap_anonymous, mprotect, syscall};

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
    let page_size =
        unsafe { NonZeroUsize::new_unchecked(syscall!(sysconf(libc::_SC_PAGE_SIZE))? as usize) };

    let ptr = mmap_anonymous(
        None,
        page_size,
        ProtFlags::PROT_READ | ProtFlags::PROT_WRITE | ProtFlags::PROT_EXEC,
        MapFlags::MAP_PRIVATE,
    )?;

    log::info!("Memory allocated at: {:p}", ptr);

    unsafe {
        // 3. copy shellcode into the allocated memory
        ptr::copy_nonoverlapping(code.as_ptr(), ptr.as_ptr() as *mut u8, code.len());
    }

    log::info!("Shellcode copied to allocated memory.");

    // 4. change memory protection to READ | EXEC
    let res = mprotect(
        ptr,
        page_size.get(),
        ProtFlags::PROT_READ | ProtFlags::PROT_EXEC,
    );

    if res.is_err() {
        unsafe {
            libc::munmap(ptr.as_ptr(), page_size.get());
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
        libc::munmap(ptr.as_ptr(), page_size.get());
    }

    Ok(())
}
