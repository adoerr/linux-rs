//!
//! This file is part of syscall-rs
//!

use std::{
    cmp, fmt,
    io::{self, IoSlice, IoSliceMut},
    mem::forget,
    os::unix::prelude::{AsRawFd, FromRawFd, IntoRawFd, RawFd},
};

/// IO system call wrapper.
///
/// Wrapper around `libc` system calls that checks `errno` on failure.
///
/// Note that we can't use the `syscall!()` macro here, because our file
/// descriptor implementation has to be compatible with `std::io` traits
/// and hence needs to return a `std::io::Result`
macro_rules! iocall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };

        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

/// According to the read(2) man page if the count of bytes to read is greater
/// than `SSIZE_MAX`, the result is implementation defined.
///
/// On Linux read(2) will transfer at most 0x7ffff000 (2,147,479,552) bytes,
/// returning the number of bytes acutally transferred
const READ_LIMIT: usize = libc::ssize_t::MAX as usize;

/// Limit for the number of buffers in the `iov` buffer array
const IOV_LIMIT: usize = libc::UIO_MAXIOV as usize;

pub struct FileDesc(RawFd);

impl FileDesc {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let res = iocall!(read(
            self.as_raw_fd(),
            buf.as_mut_ptr() as *mut libc::c_void,
            cmp::min(buf.len(), READ_LIMIT)
        ))?;

        Ok(res as usize)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
        let res = iocall!(readv(
            self.as_raw_fd(),
            bufs.as_ptr() as *const libc::iovec,
            cmp::min(bufs.len() as libc::c_int, IOV_LIMIT as libc::c_int)
        ))?;

        Ok(res as usize)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let res = iocall!(write(
            self.as_raw_fd(),
            buf.as_ptr() as *const libc::c_void,
            cmp::min(buf.len(), READ_LIMIT)
        ))?;

        Ok(res as usize)
    }

    pub fn write_vectored(&self, bufs: &mut [IoSlice]) -> io::Result<usize> {
        let res = iocall!(writev(
            self.as_raw_fd(),
            bufs.as_ptr() as *const libc::iovec,
            cmp::min(bufs.len() as libc::c_int, IOV_LIMIT as libc::c_int)
        ))?;

        Ok(res as usize)
    }
}

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
        // thread to be closed, and we don't actually really know if the fd was
        // closed or not in case of a failure.
        //
        // Also, the Linux kernel always releases the fd early in the close operation,
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
