use crate::error::{Result, ServerError};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::os::windows::io::AsRawSocket;

/// TCP listener wrapper for accepting connections
pub struct Listener {
    inner: TcpListener,
    addr: SocketAddr,
}

impl Listener {
    /// Creates a new listener bound to the given address and port
    pub fn bind(host: &str, port: u16) -> Result<Self> {
        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .map_err(|e| ServerError::Config(format!("Invalid address: {}", e)))?;

        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;

        Ok(Listener {
            inner: listener,
            addr,
        })
    }

    /// Accepts a new connection (non-blocking)
    /// Returns None if no connection is pending
    pub fn accept(&self) -> Result<Option<(TcpStream, SocketAddr)>> {
        match self.inner.accept() {
            Ok((stream, addr)) => {
                stream.set_nonblocking(true)?;
                Ok(Some((stream, addr)))
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(e) => Err(ServerError::Io(e)),
        }
    }

    /// Returns the raw socket descriptor
    pub fn fd(&self) -> u64 {
        self.inner.as_raw_socket() as u64
    }

    /// Returns the bound address
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the port
    pub fn port(&self) -> u16 {
        self.addr.port()
    }
}
