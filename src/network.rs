use serde::{Serialize, Deserialize};
use std::net::{TcpStream, TcpListener};
use std::io::Write;

/// Represents all possible messages that can be sent between players.
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Attack { x: usize, y: usize },
    Result { x: usize, y: usize, hit: bool, sunk: bool },
    Ready,
    Disconnected,
    ChatMessage(String),
}

/// Manages the network connection to the opponent.
pub struct Peer {
    pub stream: std::net::TcpStream,
}

impl Peer {
    /// Establishes a connection.
    /// If host is true, it listens for an incoming connection.
    /// If host is false, it connects to the provided address.
    pub fn new(address: &str, host: bool) -> std::io::Result<Self> {
        let stream = if host {
            // if hosting bind to a port and wait for someone to join.
            let listener = TcpListener::bind(address)?;
            let (stream, _) = listener.accept()?;
            stream
        } else {
            // if joining connect directly to the host's address.
            TcpStream::connect(address)?
        };

        // Disable Nagle's algorithm so messages are sent immediately.
        stream.set_nodelay(true)?;
        Ok(Self { stream })
    }
}