#![allow(dead_code)]

use std::{
    io::{Result, Write},
    net::TcpStream,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Request {
    delay_ms: u64,
    message: String,
}

fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:4242")?;

    let request = Request {
        delay_ms: 100,
        message: "Hello, world!".to_string(),
    };

    send(&request, &mut stream)?;

    Ok(())
}

fn send(request: &Request, stream: &mut TcpStream) -> Result<()> {
    let request = serde_json::to_string(request)?;
    stream.write_all(request.as_bytes())?;
    Ok(())
}
