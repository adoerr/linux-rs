use std::{
    io::{Read, Result},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Request {
    delay_ms: u64,
    message: String,
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4242")?;
    let (mut stream, _) = listener.accept()?;
    let request = read(&mut stream)?;
    println!("{:?}", request);
    Ok(())
}

fn read(stream: &mut TcpStream) -> Result<Request> {
    let mut buf = [0; 1024];
    let n = stream.read(&mut buf)?;
    let request = serde_json::from_slice(&buf[..n])?;
    Ok(request)
}
