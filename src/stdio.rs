//!
//! This file is part of syscall-rs
//!

use std::os::unix::prelude::RawFd;

pub enum Stdio {
    Inherit,
    Null,
    Pipe,
    Fd(RawFd),
}
