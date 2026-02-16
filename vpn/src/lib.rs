#![allow(dead_code)]

use std::net::{SocketAddrV4, UdpSocket};

use parking_lot::{Mutex, MutexGuard};
use tun_tap::Iface;

mod error;

pub use error::Result;

pub struct Peer {
    endpoint: Mutex<Option<SocketAddrV4>>,
}

pub struct Device {
    upd: UdpSocket,
    iface: Iface,
    peer: Peer,
}

impl Device {
    fn listen_iface(&self) -> Result<()> {
        let mut buf = [0u8; 1504];
        loop {
            let nbytes = self.iface.recv(&mut buf)?;
            let peer = self.peer.endpoint();
            if let Some(addr) = peer.as_ref() {
                self.upd.send_to(&buf[..nbytes], addr)?;
            }
        }
    }

    fn listen_udp(&self) -> Result<()> {
        unimplemented!()
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
