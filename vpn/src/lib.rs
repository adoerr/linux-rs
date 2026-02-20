#![allow(dead_code)]

use std::{
    net::{SocketAddr, SocketAddrV4, UdpSocket},
    sync::Arc,
};

use etherparse as parse;
use parking_lot::{MutexGuard, RwLock, RwLockReadGuard};
use socket2::{Domain, Protocol, Socket, Type};
use tun_tap::Iface;

mod error;
mod poll;

pub use error::{Error, Result};

use crate::poll::Poll;

pub struct DeviceConfig<'a> {
    use_connected_peer: bool,
    listen_port: u16,
    tun_name: &'a str,
    peer_addr: Option<SocketAddrV4>,
}

pub struct Device {
    udp: Arc<UdpSocket>,
    iface: Iface,
    peer: Peer,
    poll: Poll,
    use_connected_peer: bool,
    listen_port: u16,
}

pub struct Peer {
    endpoint: RwLock<Endpoint>,
}

#[derive(Default)]
pub struct Endpoint {
    pub addr: Option<SocketAddrV4>,
    pub conn: Option<Arc<UdpSocket>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SockID {
    Disconnected,
    ConnectedPeer,
}

fn udp_socket(port: u16) -> Result<UdpSocket> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;

    Ok(socket.into())
}

impl Device {
    pub fn new(iface: Iface, peer: Option<SocketAddrV4>) -> Device {
        let udp = udp_socket(1967).unwrap();
        Device {
            udp,
            iface,
            peer: Peer::new(Endpoint::default()),
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

    pub fn listen_udp(&self) -> Result<()> {
        let mut buf = [0u8; 1504];

        loop {
            let (nbytes, peer) = self.udp.recv_from(&mut buf)?;

            if let Ok(hdr) = parse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]) {
                let src = hdr.source_addr();
                let dst = hdr.destination_addr();
                log::debug!("got {nbytes} byte UPD packet src: {src}, dst: {dst}");
            }

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
    fn new(endpoint: Endpoint) -> Peer {
        Self {
            endpoint: RwLock::new(endpoint),
        }
    }

    fn endpoint(&self) -> RwLockReadGuard<Endpoint> {
        self.endpoint.read()
    }

    fn set_endpoint(&self, addr: SocketAddrV4) -> (bool, Option<Arc<UdpSocket>>) {
        let endpoint = self.endpoint.read();
        if endpoint.addr == Some(addr) {
            return (true, None);
        }
        drop(endpoint);

        let mut endpoint = self.endpoint.write();
        endpoint.addr = Some(addr);

        (true, endpoint.conn.take())
    }
}
