#![allow(dead_code)]

use std::{
    io::{Read, Result, Write},
    net::TcpStream,
    time::Duration,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Request {
    delay_ms: u64,
    message: String,
}

fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:4242")?;

    loop {
        let request = Request {
            delay_ms: 100,
            message: "Ping".to_string(),
        };

        send(&request, &mut stream)?;
        println!("Sent: Ping");

        let mut buf = [0; 1024];
        let n = stream.read(&mut buf)?;
        if n == 0 {
            println!("Server closed connection");
            break;
        }

        let response: Request = serde_json::from_slice(&buf[..n])?;
        println!("Received: {:?}", response.message);

        if response.message == "Pong" {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    Ok(())
}

fn send(request: &Request, stream: &mut TcpStream) -> Result<()> {
    let request = serde_json::to_string(request)?;
    stream.write_all(request.as_bytes())?;
    Ok(())
}
