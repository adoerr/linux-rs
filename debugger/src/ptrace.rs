//! Safe wrappers around `ptrace` and `waitpid` functionality.

#![allow(unused, dead_code)]

use crate::error::{Error, Result};

/// A Linux PID which is the thread ID of the corresponding thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(pub nix::unistd::Pid);

impl Pid {
    /// Create a Pid from a raw pid_t value
    pub fn from_raw(pid: libc::pid_t) -> Self {
        Self(nix::unistd::Pid::from_raw(pid))
    }
}

impl From<libc::pid_t> for Pid {
    fn from(pid: libc::pid_t) -> Self {
        Self::from_raw(pid)
    }
}

/// Ptrace operations (incomplete).
pub enum Ptrace {
    /// Indicate that this process is to be traced by its parent.
    TraceMe,

    /// Attach to an existing process, making it a tracee of the calling process.
    Attach {
        /// PID of the process to attach to.
        pid: Pid,
    },

    /// Continue the execution of a stopped process.
    Cont {
        /// PID of the process to continue.
        pid: Pid,
    },
}

/// A safe wrapper around the `ptrace` syscall.
pub fn ptrace(op: Ptrace) -> Result<()> {
    use nix::sys::{ptrace, signal::Signal};

    let result = match op {
        Ptrace::TraceMe => ptrace::traceme(),
        Ptrace::Attach { pid } => ptrace::attach(pid.0),
        Ptrace::Cont { pid } => ptrace::cont(pid.0, None), // None means no signal to deliver
    };

    result.map_err(|e| Error::IO(std::io::Error::from(e)))
}

/// Status codes from `waitpid`
#[derive(Debug)]
pub enum Status {
    /// Process terminated normally
    Exited {
        /// Least significant 8-bits of the argument to `exit`
        status: i32,
    },

    /// Processed terminated by a signal
    Signaled {
        /// Signal that caused the process to terminate
        signal: i32,

        /// Set if the child produced a core dump
        dumped: bool,
    },

    /// Child stopped by delivery of a signal
    Stopped {
        /// Signal that caused the stop
        signal: i32,

        /// Raw status code from `waitpid`
        status: i32,
    },

    /// Child resumed execution
    Continued,
}

/// A safe wrapper around the `waitpid` syscall.
pub fn waitpid(pid: Pid) -> Result<Option<Status>> {
    use nix::sys::wait::{self, WaitStatus};

    // Call waitpid without WNOHANG to make it blocking (like the original with flags=0)
    let wait_result = wait::waitpid(pid.0, None);

    match wait_result {
        Ok(wait_status) => {
            let status = match wait_status {
                WaitStatus::Exited(_pid, exit_code) => Status::Exited { status: exit_code },
                WaitStatus::Signaled(_pid, signal, core_dumped) => Status::Signaled {
                    signal: signal as i32,
                    dumped: core_dumped,
                },
                WaitStatus::Stopped(_pid, signal) => Status::Stopped {
                    signal: signal as i32,
                    status: signal as i32, // TODO: Replace signal as status with actual status code from waitpid when available. See documentation for intended behavior.
                },
                WaitStatus::Continued(_pid) => Status::Continued,
                WaitStatus::StillAlive => return Ok(None), // Shouldn't happen with blocking wait
                WaitStatus::PtraceEvent(_pid, signal, event) => Status::Stopped {
                    signal: signal as i32,
                    status: event,
                },
                WaitStatus::PtraceSyscall(_pid) => Status::Stopped {
                    signal: 0,
                    status: 0,
                },
            };
            Ok(Some(status))
        }
        Err(e) => Err(Error::IO(std::io::Error::from(e))),
    }
}
