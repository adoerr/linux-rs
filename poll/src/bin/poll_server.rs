//! A TCP server that uses `epoll` for handling concurrent connections.
//!
//! This server listens on `127.0.0.1:4242` and accepts multiple client connections.
//! It uses the `poll` library (a wrapper around `epoll`) to monitor the listening socket
//! and connected client sockets for incoming data.
//!
//! When a client sends data, the server attempts to deserialize it into a `Request` struct
//! and prints it to the console.

use std::{
    collections::HashMap,
    io::{Read, Result, Write},
    net::{TcpListener, TcpStream},
};

use poll::{Poll, poll_sys::EPOLLIN};

/// Represents a request received from a client.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Request {
    delay_ms: u64,
    message: String,
}

/// The main entry point of the server.
///
/// Initializes the TCP listener, sets up the `epoll` instance, and enters the event loop.
fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4242")?;
    listener.set_nonblocking(true)?;

    let poll = Poll::new()?;
    poll.registry.register(&listener, 0, EPOLLIN)?;

    let mut events = Vec::with_capacity(1024);
    let mut clients = HashMap::new();
    let mut token_counter = 1;

    loop {
        poll.poll(&mut events, None)?;

        for event in &events {
            let token = event.token();
            if token == 0 {
                loop {
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            println!("Incoming connection from: {addr}");
                            stream.set_nonblocking(true)?;
                            let token = token_counter;
                            token_counter += 1;
                            poll.registry.register(&stream, token, EPOLLIN)?;
                            clients.insert(token, stream);
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => return Err(e),
                    }
                }
            } else {
                if let Some(mut stream) = clients.remove(&token) {
                    match handle_client(&mut stream) {
                        Ok(_) => {
                            clients.insert(token, stream);
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            clients.insert(token, stream);
                        }
                        Err(e) => {
                            println!("Error handling client: {e:?}");
                        }
                    }
                }
            }
        }
    }
}

/// Handles an incoming request from a client stream.
///
/// Reads data from the stream, deserializes it, and prints the request.
fn handle_client(stream: &mut TcpStream) -> Result<()> {
    let request = read(stream)?;
    let addr = stream.peer_addr()?;
    println!("Received from {addr}: {request:?}");
    if request.message == "Ping" {
        let response = Request {
            delay_ms: 0,
            message: "Pong".to_string(),
        };
        let bytes = serde_json::to_vec(&response)?;
        stream.write_all(&bytes)?;
    }
    Ok(())
}

/// Reads data from the TCP stream and deserializes it into a `Request`.
///
/// This function reads up to 1024 bytes from the stream.
fn read(stream: &mut TcpStream) -> Result<Request> {
    let mut buf = [0; 1024];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Client closed connection",
        ));
    }
    let request = serde_json::from_slice(&buf[..n])?;
    Ok(request)
}
