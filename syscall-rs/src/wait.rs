//!
//! This file is part of syscall-rs
//!

use crate::{Result, Signal};

/// A [`WaitStatus`] is the result of [`wait()`]ing for a child process
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitStatus {
    /// Process exited normally with the given return code
    Exited(libc::pid_t, libc::c_int),

    /// Process was signaled by `Signal`. If a core dump was produced,
    /// the third field is `true`.
    Signaled(libc::pid_t, Signal, bool),

    /// Process is still alive, but was stopped by `Signal`
    Stopped(libc::pid_t, Signal),

    /// Process was stopped but has resumed execution after receiving a
    /// `SIGCONT` signal
    Continued(libc::pid_t),
}

impl WaitStatus {
    /// Convert a raw `status` as returned by [`libc::waitpid()`] into a [`WaitStatus`]
    ///
    /// This function is using the standard set of wait status related macros in
    /// order to dissect `status`
    pub fn from_raw(pid: libc::pid_t, status: libc::c_int) -> Result<WaitStatus> {
        Ok(if libc::WIFEXITED(status) {
            WaitStatus::Exited(pid, libc::WEXITSTATUS(status))
        } else if libc::WIFSIGNALED(status) {
            WaitStatus::Signaled(
                pid,
                libc::WTERMSIG(status).try_into()?,
                libc::WCOREDUMP(status),
            )
        } else if libc::WIFSTOPPED(status) {
            WaitStatus::Stopped(pid, libc::WSTOPSIG(status).try_into()?)
        } else {
            assert!(libc::WIFCONTINUED(status));
            WaitStatus::Continued(pid)
        })
    }
}

/// Wait for one of the children of the calling process to terminate and
/// return a [`WaitStatus`]. Note that is function will **block** until
/// a child termination status becomes available.
///
/// If `pid` is greater than `0`, wait for the child whose process ID equals
/// `pid`. In case `pid` is `None`, wait for **any** child of the calling
/// process
pub fn wait<P>(pid: P) -> Result<WaitStatus>
where
    P: Into<Option<libc::pid_t>>,
{
    let mut status: i32 = 0;

    let res = syscall!(waitpid(
        pid.into().unwrap_or(-1_i32),
        &mut status as &mut libc::c_int,
        libc::WUNTRACED
    ))?;

    WaitStatus::from_raw(res, status)
}

#[cfg(test)]
mod tests {
    use std::process::exit;

    use crate::{wait, Result, Signal, WaitStatus};

    /// This test is inherently falky and **must not** run together with other
    /// tests. Otherwise it will most likely fail by [`wait()`] returning the
    /// wait status from some random child process forked by another thread
    /// in some unrelated test instead of the wait status of `child`
    #[test]
    #[ignore = "must run in isolation"]
    fn wait_any() -> Result<()> {
        let child = match syscall!(fork())? {
            // parent
            pid if pid != 0 => pid,
            // child
            _ => {
                exit(42);
            }
        };

        if let WaitStatus::Exited(pid, status) = wait(None)? {
            assert_eq!(pid, child);
            assert_eq!(42, status);
        } else {
            // test failure
            unreachable!("wait() returned an unexpected wait status");
        }

        Ok(())
    }

    #[test]
    fn wait_exit() -> Result<()> {
        let child = match syscall!(fork())? {
            // parent
            pid if pid != 0 => pid,
            // child
            _ => {
                exit(42);
            }
        };

        if let WaitStatus::Exited(pid, status) = wait(child)? {
            assert_eq!(pid, child);
            assert_eq!(42, status);
        } else {
            // test failure
            unreachable!("wait() returned an unexpected wait status");
        }

        Ok(())
    }

    #[test]
    fn wait_stop() -> Result<()> {
        let child = match syscall!(fork())? {
            // parent
            pid if pid != 0 => pid,
            // child
            _ => loop {
                std::thread::sleep(std::time::Duration::from_millis(5));
            },
        };

        syscall!(kill(child, Signal::SIGSTOP as libc::c_int))?;

        if let WaitStatus::Stopped(pid, signal) = wait(child)? {
            assert_eq!(pid, child);
            assert_eq!(signal, Signal::SIGSTOP);
        } else {
            // test failure
            unreachable!("wait() returned an unexpected wait status");
        }

        Ok(())
    }

    #[test]
    fn wait_kill() -> Result<()> {
        let child = match syscall!(fork())? {
            // parent
            pid if pid != 0 => pid,
            // child
            _ => loop {
                std::thread::sleep(std::time::Duration::from_millis(5));
            },
        };

        syscall!(kill(child, Signal::SIGKILL as libc::c_int))?;

        if let WaitStatus::Signaled(pid, signal, core) = wait(child)? {
            assert_eq!(pid, child);
            assert_eq!(signal, Signal::SIGKILL);
            assert_eq!(core, false);
        } else {
            // test failure
            assert!(false);
        }

        Ok(())
    }

    #[test]
    fn wait_stop_kill() -> Result<()> {
        let child = match syscall!(fork())? {
            // parent
            pid if pid != 0 => pid,
            // child
            _ => loop {
                std::thread::sleep(std::time::Duration::from_millis(5));
            },
        };

        syscall!(kill(child, Signal::SIGSTOP as libc::c_int))?;

        if let WaitStatus::Stopped(pid, signal) = wait(child)? {
            assert_eq!(pid, child);
            assert_eq!(signal, Signal::SIGSTOP);
        } else {
            // test failure
            assert!(false);
        }

        syscall!(kill(child, Signal::SIGKILL as libc::c_int))?;

        if let WaitStatus::Signaled(pid, signal, core) = wait(child)? {
            assert_eq!(pid, child);
            assert_eq!(signal, Signal::SIGKILL);
            assert_eq!(core, false);
        } else {
            // test failure
            assert!(false);
        }

        Ok(())
    }

    #[test]
    fn wait_unknown() {
        let res = wait(42);

        assert_eq!(
            format!("{}", res.err().unwrap()),
            "System call error: No child processes (os error 10)"
        );
    }
}
