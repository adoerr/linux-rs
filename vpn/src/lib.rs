#![allow(dead_code)]

use std::{
    net::{SocketAddr, SocketAddrV4, UdpSocket},
    sync::Arc,
};

use parking_lot::{RwLock, RwLockReadGuard};
use socket2::{Domain, Protocol, Socket, Type};
use tun_tap::{Iface, Mode};

mod error;
mod poll;

pub use error::{Error, Result};

pub use crate::poll::Poll;
use crate::poll::Token;

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
    Invalid,
}

impl From<SockID> for i32 {
    fn from(value: SockID) -> Self {
        match value {
            SockID::Disconnected => -1,
            SockID::ConnectedPeer => 0,
            SockID::Invalid => -99,
        }
    }
}

impl From<i32> for SockID {
    fn from(value: i32) -> Self {
        match value {
            -1 => SockID::Disconnected,
            0 => SockID::ConnectedPeer,
            _ => SockID::Invalid,
        }
    }
}

const BUF_SIZE: usize = 1504;

struct ThreadData {
    msg_buf: [u8; BUF_SIZE],
}

fn udp_socket(port: u16) -> Result<UdpSocket> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;

    Ok(socket.into())
}

impl<'a> DeviceConfig<'a> {
    pub fn new(
        use_connected_peer: bool,
        listen_port: u16,
        tun_name: &'a str,
        peer_addr: Option<SocketAddrV4>,
    ) -> DeviceConfig {
        Self {
            use_connected_peer,
            listen_port,
            tun_name,
            peer_addr,
        }
    }
}

impl Device {
    pub fn new(config: DeviceConfig) -> Result<Device> {
        let iface = Iface::without_packet_info(config.tun_name, Mode::Tun)?;
        iface.set_non_blocking()?;

        let poll = Poll::new()?;
        let use_connected_peer = config.use_connected_peer;
        let listen_port = config.listen_port;

        let peer = Peer::new(Endpoint::default());
        let udp = match config.peer_addr {
            Some(addr) => {
                peer.set_endpoint(addr);
                peer.connect_endpoint(listen_port)?
            }
            None => Arc::new(udp_socket(config.listen_port)?),
        };

        Ok(Self {
            iface,
            udp,
            poll,
            peer,
            use_connected_peer,
            listen_port,
        })
    }

    pub fn wait(&self) {
        let mut t = ThreadData {
            msg_buf: [0; BUF_SIZE],
        };

        loop {
            if let Ok(token) = self.poll.wait() {
                match token {
                    Token::Tun => {
                        if let Err(err) = self.handle_tun(&mut t) {
                            log::error!("tun error: {:?}", err);
                        }
                    }
                    Token::Sock(SockID::Disconnected) => {
                        if let Err(err) = self.handle_udp(&self.udp, &mut t) {
                            log::error!("udp error: {:?}", err);
                        }
                    }
                    Token::Sock(SockID::ConnectedPeer) => {
                        if let Some(conn) = self.peer.endpoint().conn.as_deref() {
                            if let Err(err) = self.handle_peer(conn, &mut t) {
                                log::error!("peer error: {:?}", err);
                            }
                        }
                    }
                    Token::Sock(SockID::Invalid) => {
                        log::error!("invalid socket id");
                    }
                }
            } else {
                log::error!("poll wait error");
            }
        }
    }

    fn handle_tun(&self, thread_data: &mut ThreadData) -> Result<()> {
        unimplemented!()
    }

    fn handle_udp(&self, sock: &UdpSocket, thread_data: &mut ThreadData) -> Result<()> {
        unimplemented!()
    }

    fn handle_peer(&self, sock: &UdpSocket, thread_data: &mut ThreadData) -> Result<()> {
        unimplemented!()
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

    fn connect_endpoint(&self, port: u16) -> Result<Arc<UdpSocket>> {
        let mut endpoint = self.endpoint.write();
        let addr = endpoint.addr.unwrap();

        assert!(endpoint.conn.is_none());

        let conn = udp_socket(port)?;
        conn.connect(addr)?;
        let conn = Arc::new(conn);

        endpoint.conn = Some(Arc::clone(&conn));

        Ok(conn)
    }
}
