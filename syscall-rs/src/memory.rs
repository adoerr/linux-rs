//!
//! This file is part of syscall-rs
//!

use std::{num::NonZeroUsize, ptr, ptr::NonNull};

use libc::{c_int, c_void, size_t};

use crate::{Error, Result, libc_bitflags};

libc_bitflags! {
    /// Desired memory protection of a memory mapping.
    pub struct ProtFlags: c_int {
        /// Pages cannot be accessed.
        PROT_NONE;
        /// Pages can be read.
        PROT_READ;
        /// Pages can be written.
        PROT_WRITE;
        /// Pages can be executed
        PROT_EXEC;
    }
}

libc_bitflags! {
    /// Additional parameters for [`mmap`].
    pub struct MapFlags: c_int {
        /// Compatibility flag. Ignored.
        MAP_FILE;
        /// Share this mapping. Mutually exclusive with `MAP_PRIVATE`.
        MAP_SHARED;
        /// Force mmap to check and fail on unknown flags. This also enables `MAP_SYNC`.
        MAP_SHARED_VALIDATE;
        /// Create a private copy-on-write mapping. Mutually exclusive with `MAP_SHARED`.
        MAP_PRIVATE;
        /// Place the mapping at exactly the address specified in `addr`.
        MAP_FIXED;
        /// Place the mapping at exactly the address specified in `addr`, but never clobber an existing range.
        MAP_FIXED_NOREPLACE;
        /// Synonym for `MAP_ANONYMOUS`.
        MAP_ANON;
        /// The mapping is not backed by any file.
        MAP_ANONYMOUS;
        /// Do not reserve swap space for this mapping.
        MAP_NORESERVE;
        /// Make use of 64KB huge page (must be supported by the system)
        MAP_HUGE_64KB;
        /// Make use of 512KB huge page (must be supported by the system)
        MAP_HUGE_512KB;
        /// Make use of 1MB huge page (must be supported by the system)
        MAP_HUGE_1MB;
        /// Make use of 2MB huge page (must be supported by the system)
        MAP_HUGE_2MB;
        /// Make use of 8MB huge page (must be supported by the system)
        MAP_HUGE_8MB;
        /// Make use of 16MB huge page (must be supported by the system)
        MAP_HUGE_16MB;
        /// Make use of 32MB huge page (must be supported by the system)
        MAP_HUGE_32MB;
        /// Make use of 256MB huge page (must be supported by the system)
        MAP_HUGE_256MB;
        /// Make use of 512MB huge page (must be supported by the system)
        MAP_HUGE_512MB;
        /// Make use of 1GB huge page (must be supported by the system)
        MAP_HUGE_1GB;
        /// Make use of 2GB huge page (must be supported by the system)
        MAP_HUGE_2GB;
        /// Make use of 16GB huge page (must be supported by the system)
        MAP_HUGE_16GB;
        /// Do not write through the page caches, write directly to the file. Used for Direct Access (DAX) enabled file systems.
        MAP_SYNC;
    }
}

/// Creates an anonymous memory mapping
pub fn mmap_anonymous(
    addr: Option<NonZeroUsize>,
    len: NonZeroUsize,
    prot: ProtFlags,
    flags: MapFlags,
) -> Result<NonNull<c_void>> {
    const NO_FILE_DESCRIPTOR: c_int = -1;

    let ptr = addr.map_or(ptr::null_mut(), |a| a.get() as *mut c_void);

    let flags = MapFlags::MAP_ANONYMOUS | flags;
    let ret = unsafe {
        libc::mmap(
            ptr,
            len.into(),
            prot.bits(),
            flags.bits(),
            NO_FILE_DESCRIPTOR,
            0,
        )
    };

    if ptr::eq(ret, libc::MAP_FAILED) {
        Err(Error::Syscall(std::io::Error::last_os_error()))
    } else {
        // SAFETY: `libc::mmap` returns a valid non-null pointer or `libc::MAP_FAILED`, thus `ret`
        // will be non-null here.
        Ok(unsafe { NonNull::new_unchecked(ret) })
    }
}

/// Changes the memory protection of a given memory region.
pub fn mprotect(addr: NonNull<c_void>, len: size_t, prot: ProtFlags) -> Result<()> {
    syscall!(mprotect(addr.as_ptr(), len, prot.bits())).map(|_| ())
}

/// Unmaps a previously mapped memory region.
pub fn munmap(addr: NonNull<c_void>, len: size_t) -> Result<()> {
    syscall!(munmap(addr.as_ptr(), len)).map(|_| ())
}
