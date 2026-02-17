#![allow(dead_code)]

use std::net::{SocketAddr, SocketAddrV4, UdpSocket};

use parking_lot::{Mutex, MutexGuard};
use tun_tap::Iface;

mod error;

pub use error::Result;

pub struct Peer {
    endpoint: Mutex<Option<SocketAddrV4>>,
}

pub struct Device {
    udp: UdpSocket,
    iface: Iface,
    peer: Peer,
}

impl Device {
    fn listen_iface(&self) -> Result<()> {
        let mut buf = [0u8; 1504];
        {
            let peer = self.peer.endpoint();
            if let Some(addr) = peer.as_ref() {
                log::debug!("handshake to: {addr:?}");
                self.udp.send_to("hello?".as_bytes(), addr)?;
            }
        }
        loop {
            let nbytes = self.iface.recv(&mut buf)?;
            let peer = self.peer.endpoint();
            if let Some(addr) = peer.as_ref() {
                self.udp.send_to(&buf[..nbytes], addr)?;
            }
        }
    }

    fn listen_udp(&self) -> Result<()> {
        let mut buf = [0u8; 1504];
        loop {
            let (nbytes, peer) = self.udp.recv_from(&mut buf)?;
            if let SocketAddr::V4(addr) = peer {
                if &buf[..nbytes] == b"hello?" {
                    log::debug!("handshake from: {addr:?}");
                    self.peer.set_endpoint(addr);
                    continue;
                }
                self.iface.send(&buf[..nbytes])?;
            }
        }
    }
}

impl Peer {
    fn endpoint(&self) -> MutexGuard<'_, Option<SocketAddrV4>> {
        self.endpoint.lock()
    }

    fn set_endpoint(&self, addr: SocketAddrV4) {
        let mut endpoint = self.endpoint.lock();
        if endpoint.is_none() {
            *endpoint = Some(addr);
        }
    }
}
