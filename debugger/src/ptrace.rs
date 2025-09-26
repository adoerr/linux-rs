//! Safe wrappers around `ptrace` and `waitpid` functionality.

#![allow(unused, dead_code)]

use crate::error::{Error, Result};

/// A Linux PID which is the thread ID of the corresponding thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Pid(libc::pid_t);

/// Ptrace operations (incomplete).
pub(crate) enum Ptrace {
    /// Indicate that this process is to be traced by its parent.
    TraceMe,

    /// Attach to an existing process, making it a tracee of the calling process.
    Attach {
        /// PID of the process to attach to.
        pid: Pid,
    },
}

/// A safe wrapper around the `ptrace` syscall.
pub(crate) fn ptrace(op: Ptrace) -> Result<()> {
    let res = match op {
        Ptrace::TraceMe => unsafe { libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) },
        Ptrace::Attach { pid } => unsafe { libc::ptrace(libc::PTRACE_ATTACH, pid.0, 0, 0) },
    };

    if res == -1 {
        return Err(Error::IO(std::io::Error::last_os_error()));
    }

    Ok(())
}

/// Status codes from `waitpid`
#[derive(Debug)]
pub(crate) enum Status {
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
pub(crate) fn waitpid(pid: Pid) -> Result<Option<Status>> {
    // check the status on `pid`, non-blocking
    let mut status = 0;

    let ret = unsafe { libc::waitpid(pid.0, &mut status, libc::WNOHANG | libc::WCONTINUED) };

    // check return value
    if ret == 0 {
        // No status
        return Ok(None);
    } else if ret == -1 {
        // Error
        return Err(Error::IO(std::io::Error::last_os_error()));
    }

    // convert status
    let status = match () {
        _ if libc::WIFEXITED(status) => Status::Exited {
            status: libc::WEXITSTATUS(status),
        },
        _ if libc::WIFSIGNALED(status) => Status::Signaled {
            signal: libc::WTERMSIG(status),
            dumped: libc::WCOREDUMP(status),
        },
        _ if libc::WIFSTOPPED(status) => Status::Stopped {
            signal: libc::WSTOPSIG(status),
            status,
        },
        _ if libc::WIFCONTINUED(status) => Status::Continued,
        _ => unreachable!("Unknown waitpid() status"),
    };

    Ok(Some(status))
}
