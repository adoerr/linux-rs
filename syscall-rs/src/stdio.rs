//!
//! This file is part of syscall-rs
//!

use crate::FileDesc;

pub enum Stdio {
    Fd(FileDesc),
    Inherit,
    Null,
    Pipe,
}
