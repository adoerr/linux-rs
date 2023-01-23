//!
//! This file is part of syscall-rs
//!

fn main() {
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    compile_error!("this crate is Linux only!");
}
