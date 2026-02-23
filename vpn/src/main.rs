#![allow(dead_code)]
#![allow(unused_imports)]

use std::{net::SocketAddr, sync::Arc, thread};

use argh::FromArgs;
use tun_tap::{Iface, Mode};
use vpn::{Device, DeviceConfig, Peer, Result};

/// Test VPN command
#[derive(FromArgs, PartialEq, Debug)]
struct Cli {
    #[argh(positional)]
    peer: Option<String>,
}

fn main() -> Result<()> {
    env_logger::init();
    log::info!("starting vpn ...");

    let cli: Cli = argh::from_env();

    run(cli.peer.as_deref())?;

    Ok(())
}

fn run(addr: Option<&str>) -> Result<()> {
    let peer = addr
        .and_then(|addr| addr.parse().ok())
        .and_then(|addr| match addr {
            SocketAddr::V4(v4) => Some(v4),
            _ => None,
        });

    let conf = DeviceConfig::new(peer.is_none(), 1967, "vpn0", peer);
    let dev = Device::new(conf)?;
    dev.start()?;
    dev.wait();

    Ok(())
}
