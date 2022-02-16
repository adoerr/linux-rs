//
// This example demonstrates how to process signals and termination status
// from a forked child process.
//
// Run this example using
//
// RUST_LOG=info cargo run --example wait_for_child
//

use anyhow::bail;
use log::info;
use std::{process::exit, thread::sleep, time::Duration};
use syscall::{signal_block, syscall, wait, Signal, SignalFd};

fn main() -> anyhow::Result<()> {
    env_logger::try_init()?;

    // We need to block the signals that we want to fetch using a signal fd.
    // This will prevent those signals from being handled by other means, like
    // a signal handler etc.
    signal_block(vec![Signal::SIGCHLD].as_slice().into())?;

    info!("about to fork child");

    let child = match syscall!(fork())? {
        // parent -> child pid
        pid if pid != 0 => pid,
        // child -> quick nap, then exit
        _ => {
            info!("child is a go");

            sleep(Duration::from_millis(20));

            info!("child is about to exit");

            exit(42);
        }
    };

    info!("wait for child `{}`", child);

    let mut sigfd = SignalFd::new(vec![Signal::SIGCHLD].as_slice().into())?;

    match sigfd.read_signal()? {
        Signal::SIGCHLD => {
            info!("got SIGCHLD - fetch child termination status");

            let status = wait(child)?;

            info!("child termination status `{:?}`", status);
        }
        sig => {
            bail!("received invalid signal `{}`", sig);
        }
    }

    Ok(())
}
