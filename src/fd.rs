//!
//! This file is part of syscall-rs
//!

use std::{
    fmt,
    mem::forget,
    os::unix::prelude::{AsRawFd, FromRawFd, IntoRawFd, RawFd},
};

pub struct FileDesc(RawFd);

impl AsRawFd for FileDesc {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl IntoRawFd for FileDesc {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.0;
        forget(self);
        fd
    }
}

impl FromRawFd for FileDesc {
    /// Return a [`FileDesc`] form a [`RawFd`]
    ///
    /// ### Safety
    ///
    /// The resource pointed to by `fd` must be open and must not require any
    /// cleanup other than `close(2)`
    ///
    /// This function will assert that `fd` is in the valid range and is not `-1`
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        assert_ne!(fd, u32::MAX as RawFd);
        Self(fd)
    }
}

impl Drop for FileDesc {
    fn drop(&mut self) {
        // Note that errors are ignored because retrying the close after a failure
        // is the wrong thing to do, since this may cause a reused fd from another
        // thread to be closed and we don't acutally really know if the fd was
        // closed or not in case of a failure.
        //
        // Also the Linux kernel always releases the fd early in the close operation,
        // freeing it for reuse.
        unsafe {
            let _ = libc::close(self.0);
        }
    }
}

impl fmt::Debug for FileDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FileDesc").field(&self.0).finish()
    }
}
