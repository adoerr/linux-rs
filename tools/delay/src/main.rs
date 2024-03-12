use std::{
    io::{Read, Result},
    net::{SocketAddr, TcpListener, TcpStream},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Request {
    delay_ms: u64,
    message: String,
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4242")?;

    while let Ok((stream, addr)) = listener.accept() {
        handle_client(stream, addr)?;
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, addr: SocketAddr) -> Result<()> {
    println!("Incoming connection from: {addr}");
    let request = read(&mut stream)?;
    println!("{request:?}");
    Ok(())
}

fn read(stream: &mut TcpStream) -> Result<Request> {
    let mut buf = [0; 1024];
    let n = stream.read(&mut buf)?;
    let request = serde_json::from_slice(&buf[..n])?;
    Ok(request)
}
