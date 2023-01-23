//! Safe wrappers around [`libc::ptrace`]

use crate::error::{Error, Result};

/// Linux PID. Each thread is a PID.
#[derive(Clone, Copy, Debug, PartialOrd, Ord, Eq, PartialEq)]
pub(crate) struct Pid(pub(crate) libc::pid_t);

/// Ptrace action to be performed
pub(crate) enum PtraceAction {
    /// Indicate that this process is to be traced by its parent
    TraceMe,
}

/// Safe wrapper around the `ptrace` syscall
pub(crate) fn ptrace(action: PtraceAction) -> Result<()> {
    let res = match action {
        PtraceAction::TraceMe => unsafe { libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) },
    };

    if res != -1 {
        Ok(())
    } else {
        Err(Error::IO(std::io::Error::last_os_error()))
    }
}
