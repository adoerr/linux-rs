//!
//! This file is part of syscall-rs
//!

#![deny(clippy::all)]
#![allow(clippy::macro_metavars_in_unsafe)]

/// System call wrapper.
///
/// Wrapper around `libc` system calls that checks `errno` on failure.
#[macro_export]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };

        if res == -1 {
            Err($crate::Error::Syscall(std::io::Error::last_os_error()))
        } else {
            Ok(res)
        }
    }};
}

mod error;
mod fd;
mod signal;
mod stdio;
mod wait;

pub use error::{Error, Result};
pub use fd::FileDesc;
pub use signal::{Signal, SignalFd, SignalSet, signal_block, signal_restore};
pub use stdio::Stdio;
pub use wait::{WaitStatus, wait};
