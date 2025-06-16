mod error;
mod ptrace;

pub use error::{Error, Result};
pub use ptrace::{Pid, PtraceAction, ptrace};
