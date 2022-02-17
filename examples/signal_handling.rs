//!
//! This file is part of syscall-rs
//!

use anyhow::Result;
use log::info;
use mio::{Events, Interest, Poll, Token};
use std::process;
use syscall::{signal_block, Signal, SignalFd, SignalSet};

const SIGNAL: Token = Token(42);

fn main() -> Result<()> {
    env_logger::try_init()?;

    signal_block(SignalSet::fill()?)?;

    let mut poll = Poll::new()?;

    let mut sigfd = SignalFd::new(SignalSet::fill()?)?;

    poll.registry()
        .register(&mut sigfd, SIGNAL, Interest::READABLE)?;

    println!("Send `kill -s TERM {}` to stop the process", process::id());

    let mut events = Events::with_capacity(3);

    loop {
        poll.poll(&mut events, None)?;

        for evt in events.iter() {
            match evt.token() {
                SIGNAL => match sigfd.read_signal()? {
                    Signal::SIGTERM => {
                        info!("got signal TERM");
                        return Ok(());
                    }
                    sig => {
                        info!("got signal `{:?}`", sig);
                    }
                },
                _ => {
                    info!("unexpected event `{:?}`", evt);
                    break;
                }
            }
        }
    }
}
