use crate::error::{Result, ServerError};
use crate::http::{Request, RequestParser, Response};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::windows::io::AsRawSocket;
use std::time::Instant;

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Reading request data
    Reading,
    /// Processing request
    Processing,
    /// Writing response data
    Writing,
    /// Connection should be closed
    Closed,
}

/// Represents a client connection
pub struct Connection {
    /// The TCP stream
    pub stream: TcpStream,
    /// Client address
    pub addr: SocketAddr,
    /// Server port this connection came from
    pub server_port: u16,
    /// Current state
    pub state: ConnectionState,
    /// Read buffer
    pub read_buffer: Vec<u8>,
    /// Write buffer
    pub write_buffer: Vec<u8>,
    /// Bytes written so far
    pub bytes_written: usize,
    /// When the connection was created
    pub created_at: Instant,
    /// Last activity time
    pub last_activity: Instant,
    /// Keep connection alive
    pub keep_alive: bool,
}

impl Connection {
    /// Creates a new connection
    pub fn new(stream: TcpStream, addr: SocketAddr, server_port: u16) -> Self {
        let now = Instant::now();
        Connection {
            stream,
            addr,
            server_port,
            state: ConnectionState::Reading,
            read_buffer: Vec::with_capacity(8192),
            write_buffer: Vec::new(),
            bytes_written: 0,
            created_at: now,
            last_activity: now,
            keep_alive: true,
        }
    }

    /// Returns the raw socket descriptor
    pub fn fd(&self) -> u64 {
        self.stream.as_raw_socket() as u64
    }

    /// Reads available data from the socket (non-blocking)
    pub fn read(&mut self) -> Result<usize> {
        let mut buf = [0u8; 8192];
        
        match self.stream.read(&mut buf) {
            Ok(0) => {
                // Connection closed by peer
                self.state = ConnectionState::Closed;
                Ok(0)
            }
            Ok(n) => {
                self.read_buffer.extend_from_slice(&buf[..n]);
                self.last_activity = Instant::now();
                Ok(n)
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(0)
            }
            Err(e) => {
                self.state = ConnectionState::Closed;
                Err(ServerError::Io(e))
            }
        }
    }

    /// Writes data to the socket (non-blocking)
    pub fn write(&mut self) -> Result<usize> {
        if self.bytes_written >= self.write_buffer.len() {
            return Ok(0);
        }

        let remaining = &self.write_buffer[self.bytes_written..];
        
        match self.stream.write(remaining) {
            Ok(0) => {
                self.state = ConnectionState::Closed;
                Ok(0)
            }
            Ok(n) => {
                self.bytes_written += n;
                self.last_activity = Instant::now();

                // Check if we've written everything
                if self.bytes_written >= self.write_buffer.len() {
                    if self.keep_alive {
                        // Reset for next request
                        self.read_buffer.clear();
                        self.write_buffer.clear();
                        self.bytes_written = 0;
                        self.state = ConnectionState::Reading;
                    } else {
                        self.state = ConnectionState::Closed;
                    }
                }

                Ok(n)
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(0)
            }
            Err(e) => {
                self.state = ConnectionState::Closed;
                Err(ServerError::Io(e))
            }
        }
    }

    /// Checks if we have a complete request
    pub fn has_complete_request(&self) -> bool {
        RequestParser::is_complete(&self.read_buffer)
    }

    /// Parses the request from the read buffer
    pub fn parse_request(&self) -> Result<Request> {
        RequestParser::parse(&self.read_buffer)
    }

    /// Sets the response to send
    pub fn set_response(&mut self, response: Response) {
        self.write_buffer = response.to_bytes();
        self.bytes_written = 0;
        self.state = ConnectionState::Writing;
        self.keep_alive = response.headers.keep_alive();
    }

    /// Checks if the connection has timed out
    pub fn is_timed_out(&self, timeout_secs: u64) -> bool {
        self.last_activity.elapsed().as_secs() > timeout_secs
    }

    /// Checks if writing is complete
    pub fn is_write_complete(&self) -> bool {
        self.bytes_written >= self.write_buffer.len()
    }
}
