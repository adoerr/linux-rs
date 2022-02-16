//
// This example demonstrates how to process signals and termination status
// from a forked child process.
//

use anyhow::bail;
use std::{process::exit, thread::sleep, time::Duration};
use syscall::{signal_block, syscall, wait, Signal, SignalFd};

fn main() -> anyhow::Result<()> {
    // We need to block the signals that we want to fetch using a signal fd.
    // This will prevent those signals from being handled by other means, like
    // a signal handler etc.
    signal_block(
        vec![
            Signal::SIGCHLD,
            Signal::SIGINT,
            Signal::SIGQUIT,
            Signal::SIGTERM,
        ]
        .as_slice()
        .into(),
    )?;

    println!("about to fork child");

    let child = match syscall!(fork())? {
        // parent -> child pid
        pid if pid != 0 => pid,
        // child -> quick nap, then exit
        _ => {
            println!("child is a go");

            sleep(Duration::from_millis(20));

            println!("child is about to exit");

            exit(42);
        }
    };

    println!("wait for child `{}`", child);

    let mut sigfd = SignalFd::new(
        vec![
            Signal::SIGCHLD,
            Signal::SIGINT,
            Signal::SIGQUIT,
            Signal::SIGTERM,
        ]
        .as_slice()
        .into(),
    )?;

    match sigfd.read_signal()? {
        Signal::SIGCHLD => {
            println!("got SIGCHLD - fetch child termination status");

            let status = wait(child)?;

            println!("child termination status `{:?}`", status);
        }
        sig => {
            bail!("received invalid signal `{}`", sig);
        }
    }

    Ok(())
}
