//! Safe wrappers around `ptrace` and `waitpid` functionality.

#![allow(unused, dead_code)]

/// A Linux PID which is the thread ID of the corresponding thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Pid(libc::pid_t);

/// Ptrace operations (incomplete).
pub(crate) enum Ptrace {
    // Indicate that this process is to be traced by its parent.
    TraceMe,

    // Attach to an existing process, making it a tracee of the calling process.
    Attach {
        /// PID of the process to attach to.
        pid: Pid,
    },
}
