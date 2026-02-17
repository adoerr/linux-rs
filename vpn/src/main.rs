#![allow(dead_code)]
#![allow(unused_imports)]

use std::net::SocketAddr;

use argh::FromArgs;
use tun_tap::{Iface, Mode};
use vpn::{Device, Peer, Result};

/// Test VPN command
#[derive(FromArgs, PartialEq, Debug)]
struct Cli {
    #[argh(positional)]
    peer: Option<String>,
}

fn main() -> Result<()> {
    env_logger::init();
    log::info!("starting vpn ...");

    let _cli: Cli = argh::from_env();

    Ok(())
}

fn run(addr: Option<&str>) -> Result<()> {
    let _iface = Iface::without_packet_info("vpn0", Mode::Tun)?;

    let _peer = addr.and_then(|addr| addr.parse().ok()).and_then(|addr| {
        if let SocketAddr::V4(addr) = addr {
            Some(addr)
        } else {
            None
        }
    });

    Ok(())
}
