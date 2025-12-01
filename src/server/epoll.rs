use crate::error::{Result, ServerError};
use std::collections::HashMap;
use std::net::TcpStream;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::io::AsRawSocket;

#[cfg(not(windows))]
use std::os::unix::io::AsRawFd;

/// Event types for polling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Read,
    Write,
    ReadWrite,
}

/// An event returned by the poller
#[derive(Debug, Clone)]
pub struct Event {
    pub fd: u64,
    pub readable: bool,
    pub writable: bool,
}

/// Cross-platform event poller
/// Uses a simple polling approach compatible with Windows and Unix
pub struct Poller {
    /// Registered file descriptors and their event types
    registered: HashMap<u64, EventType>,
}

impl Poller {
    /// Creates a new Poller
    pub fn new() -> Result<Self> {
        Ok(Poller {
            registered: HashMap::new(),
        })
    }

    /// Registers a socket for events
    pub fn register(&mut self, fd: u64, event_type: EventType) -> Result<()> {
        self.registered.insert(fd, event_type);
        Ok(())
    }

    /// Modifies the event type for a registered socket
    pub fn modify(&mut self, fd: u64, event_type: EventType) -> Result<()> {
        if self.registered.contains_key(&fd) {
            self.registered.insert(fd, event_type);
            Ok(())
        } else {
            Err(ServerError::Internal("Socket not registered".to_string()))
        }
    }

    /// Unregisters a socket
    pub fn unregister(&mut self, fd: u64) -> Result<()> {
        self.registered.remove(&fd);
        Ok(())
    }

    /// Waits for events with a timeout
    /// Returns a list of events that occurred
    pub fn wait(&mut self, timeout: Option<Duration>) -> Result<Vec<Event>> {
        if self.registered.is_empty() {
            if let Some(t) = timeout {
                std::thread::sleep(t.min(Duration::from_millis(100)));
            }
            return Ok(Vec::new());
        }

        // Simple polling: sleep briefly and return all registered as ready
        // This is a simplified approach - in production you'd use platform-specific APIs
        if let Some(t) = timeout {
            std::thread::sleep(t.min(Duration::from_millis(10)));
        }

        // Return all registered sockets as potentially ready
        // The actual I/O operations will handle EWOULDBLOCK
        let events: Vec<Event> = self.registered
            .iter()
            .map(|(&fd, &event_type)| Event {
                fd,
                readable: matches!(event_type, EventType::Read | EventType::ReadWrite),
                writable: matches!(event_type, EventType::Write | EventType::ReadWrite),
            })
            .collect();

        Ok(events)
    }

    /// Helper to get raw socket from TcpStream
    #[cfg(windows)]
    pub fn get_fd(stream: &TcpStream) -> u64 {
        stream.as_raw_socket() as u64
    }

    #[cfg(not(windows))]
    pub fn get_fd(stream: &TcpStream) -> u64 {
        stream.as_raw_fd() as u64
    }
}

impl Default for Poller {
    fn default() -> Self {
        Self::new().expect("Failed to create poller")
    }
}

/// Sets a socket to non-blocking mode
pub fn set_nonblocking(stream: &TcpStream) -> Result<()> {
    stream.set_nonblocking(true)?;
    Ok(())
}

/// Sets TCP_NODELAY on a socket
pub fn set_nodelay(stream: &TcpStream) -> Result<()> {
    stream.set_nodelay(true)?;
    Ok(())
}
