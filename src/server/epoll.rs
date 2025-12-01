use crate::error::{Result, ServerError};
use std::collections::HashMap;
use std::io;
use std::net::TcpStream;
use std::os::windows::io::AsRawSocket;
use std::time::Duration;

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

/// Cross-platform event poller (uses select on Windows)
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
    pub fn wait(&self, timeout: Option<Duration>) -> Result<Vec<Event>> {
        if self.registered.is_empty() {
            // Sleep briefly if nothing to poll
            if let Some(t) = timeout {
                std::thread::sleep(t.min(Duration::from_millis(100)));
            }
            return Ok(Vec::new());
        }

        let mut events = Vec::new();

        // Use select-based polling on Windows
        unsafe {
            let mut read_fds: libc::fd_set = std::mem::zeroed();
            let mut write_fds: libc::fd_set = std::mem::zeroed();

            // Initialize fd_sets
            libc::FD_ZERO(&mut read_fds);
            libc::FD_ZERO(&mut write_fds);

            let mut max_fd: i32 = 0;

            for (&fd, &event_type) in &self.registered {
                let socket_fd = fd as i32;
                max_fd = max_fd.max(socket_fd);

                match event_type {
                    EventType::Read => {
                        libc::FD_SET(socket_fd as libc::SOCKET, &mut read_fds);
                    }
                    EventType::Write => {
                        libc::FD_SET(socket_fd as libc::SOCKET, &mut write_fds);
                    }
                    EventType::ReadWrite => {
                        libc::FD_SET(socket_fd as libc::SOCKET, &mut read_fds);
                        libc::FD_SET(socket_fd as libc::SOCKET, &mut write_fds);
                    }
                }
            }

            // Set timeout
            let mut tv = libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };

            let timeout_ptr = if let Some(t) = timeout {
                tv.tv_sec = t.as_secs() as i32;
                tv.tv_usec = t.subsec_micros() as i32;
                &mut tv as *mut libc::timeval
            } else {
                std::ptr::null_mut()
            };

            // Call select
            let result = libc::select(
                max_fd + 1,
                &mut read_fds,
                &mut write_fds,
                std::ptr::null_mut(),
                timeout_ptr,
            );

            if result < 0 {
                let err = io::Error::last_os_error();
                // Ignore interrupted system calls
                if err.kind() != io::ErrorKind::Interrupted {
                    return Err(ServerError::Io(err));
                }
                return Ok(Vec::new());
            }

            // Check which sockets have events
            for &fd in self.registered.keys() {
                let socket_fd = fd as libc::SOCKET;
                let readable = libc::FD_ISSET(socket_fd, &read_fds);
                let writable = libc::FD_ISSET(socket_fd, &write_fds);

                if readable || writable {
                    events.push(Event {
                        fd,
                        readable,
                        writable,
                    });
                }
            }
        }

        Ok(events)
    }

    /// Helper to get raw socket from TcpStream
    pub fn get_fd(stream: &TcpStream) -> u64 {
        stream.as_raw_socket() as u64
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
