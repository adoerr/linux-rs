#![allow(dead_code)]
#![allow(unused_imports)]

use std::{net::SocketAddr, sync::Arc, thread};

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

    let cli: Cli = argh::from_env();

    run(cli.peer.as_deref())?;

    Ok(())
}

fn run(addr: Option<&str>) -> Result<()> {
    let iface = Iface::without_packet_info("vpn0", Mode::Tun)?;

    let peer = addr
        .and_then(|addr| addr.parse().ok())
        .and_then(|addr| match addr {
            SocketAddr::V4(v4) => Some(v4),
            _ => None,
        });

    let dev = Device::new(iface, peer);
    let dev1 = Arc::new(dev);
    let dev2 = Arc::clone(&dev1);

    let thrd1 = thread::spawn(move || -> Result<()> { dev1.listen_iface() });
    let thrd2 = thread::spawn(move || -> Result<()> { dev2.listen_udp() });

    _ = thrd1.join().unwrap();
    _ = thrd2.join().unwrap();

    Ok(())
}
