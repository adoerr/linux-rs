//!
//! This file is part of syscall-rs
//!

use anyhow::{bail, Result};
use syscall::{signal_block, syscall, wait, Signal, SignalFd, WaitStatus};

#[test]
fn wait_child() -> Result<()> {
    // don't handle signal the usual way
    signal_block(vec![Signal::SIGCHLD].as_slice().into())?;

    let child = match syscall!(fork())? {
        // parent -> child pid
        pid if pid != 0 => pid,
        // child -> exit
        _ => {
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
