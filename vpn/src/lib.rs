#![allow(dead_code)]

use std::net::{SocketAddr, SocketAddrV4, UdpSocket};

use etherparse as parse;
use parking_lot::{Mutex, MutexGuard};
use socket2::{Domain, Protocol, Socket, Type};
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

fn udp_socket(port: u16) -> Result<UdpSocket> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;

    Ok(socket.into())
}

impl Device {
    pub fn new(iface: Iface, peer: Option<SocketAddrV4>) -> Device {
        let udp = udp_socket(1967).unwrap();
        Device {
            udp,
            iface,
            peer: Peer {
                endpoint: Mutex::new(peer),
            },
        }
    }

    pub fn listen_iface(&self) -> Result<()> {
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

            if let Ok(hdr) = parse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]) {
                let src = hdr.source_addr();
                let dst = hdr.destination_addr();
                log::debug!("got {nbytes} byte IPv4 packet src: {src}, dst: {dst}");
            }

            let peer = self.peer.endpoint();

            if let Some(addr) = peer.as_ref() {
                self.udp.send_to(&buf[..nbytes], addr)?;
            } else {
                log::error!("no peer found");
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
