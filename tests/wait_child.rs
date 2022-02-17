//!
//! This file is part of syscall-rs
//!

use anyhow::{bail, Result};
use syscall::{signal_block, syscall, wait, Signal, SignalFd, WaitStatus};

#[test]
fn wait_child() -> Result<()> {
    // make sure, we do get SIGCHILD
    syscall!(prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0))?;

    // don't handle SIGCHILD the usual way
    signal_block(vec![Signal::SIGCHLD].as_slice().into())?;

    let child = match syscall!(fork())? {
        // parent -> child pid
        pid if pid != 0 => pid,
        // child -> quick nap then exit
        _ => {
            std::thread::sleep(std::time::Duration::from_millis(5));

            std::process::exit(42);
        }
    };

    let mut sigfd = SignalFd::new(vec![Signal::SIGCHLD].as_slice().into())?;

    match sigfd.read_signal()? {
        Signal::SIGCHLD => match wait(child)? {
            WaitStatus::Exited(pid, status) => {
                assert_eq!(child, pid);
                assert_eq!(42, status);
            }
            _ => {
                bail!("unexpected wait status");
            }
        },
        _ => {
            bail!("unexpected signal");
        }
    }

    Ok(())
}
