use super::connection::{Connection, ConnectionState};
use super::epoll::{EventType, Poller};
use super::listener::Listener;
use crate::config::Config;
use crate::error::Result;
use crate::http::Response;
use crate::router::Handler;
use std::collections::HashMap;
use std::time::Duration;

/// Main event loop for the server
pub struct EventLoop {
    /// Configuration
    config: Config,
    /// Event poller
    poller: Poller,
    /// TCP listeners (fd -> Listener)
    listeners: HashMap<u64, Listener>,
    /// Active connections (fd -> Connection)
    connections: HashMap<u64, Connection>,
    /// Request handler
    handler: Handler,
    /// Running flag
    running: bool,
}

impl EventLoop {
    /// Creates a new event loop with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        let poller = Poller::new()?;
        let handler = Handler::new(config.clone());

        Ok(EventLoop {
            config,
            poller,
            listeners: HashMap::new(),
            connections: HashMap::new(),
            handler,
            running: false,
        })
    }

    /// Starts the event loop
    pub fn run(&mut self) -> Result<()> {
        self.running = true;

        // Create listeners for all configured addresses
        for (host, port) in self.config.get_listen_addresses() {
            let listener = Listener::bind(&host, port)?;
            let fd = listener.fd();
            self.poller.register(fd, EventType::Read)?;
            self.listeners.insert(fd, listener);
        }

        println!("Server started, waiting for connections...");

        // Main event loop
        while self.running {
            // Poll for events with 100ms timeout
            let events = self.poller.wait(Some(Duration::from_millis(100)))?;

            for event in events {
                if self.listeners.contains_key(&event.fd) {
                    // New connection on a listener
                    if event.readable {
                        self.accept_connection(event.fd)?;
                    }
                } else if self.connections.contains_key(&event.fd) {
                    // Get connection state first
                    let state = self.connections.get(&event.fd)
                        .map(|c| c.state)
                        .unwrap_or(ConnectionState::Closed);
                    
                    // Handle based on state
                    if event.readable && state == ConnectionState::Reading {
                        self.handle_read(event.fd)?;
                    }
                    if event.writable && state == ConnectionState::Writing {
                        self.handle_write(event.fd)?;
                    }
                }
            }

            // Check for timeouts and process pending requests
            self.process_connections()?;
        }

        Ok(())
    }

    /// Accepts a new connection
    fn accept_connection(&mut self, listener_fd: u64) -> Result<()> {
        let listener = match self.listeners.get(&listener_fd) {
            Some(l) => l,
            None => return Ok(()),
        };

        let port = listener.port();

        // Accept all pending connections
        while let Some((stream, addr)) = listener.accept()? {
            let conn = Connection::new(stream, addr, port);
            let fd = conn.fd();

            // Register for read events
            self.poller.register(fd, EventType::Read)?;
            self.connections.insert(fd, conn);
        }

        Ok(())
    }

    /// Handles readable event on a connection
    fn handle_read(&mut self, fd: u64) -> Result<()> {
        let conn = match self.connections.get_mut(&fd) {
            Some(c) => c,
            None => return Ok(()),
        };

        // Read available data
        conn.read()?;

        // Check if we have a complete request
        if conn.has_complete_request() {
            conn.state = ConnectionState::Processing;
        }

        Ok(())
    }

    /// Handles writable event on a connection
    fn handle_write(&mut self, fd: u64) -> Result<()> {
        let conn = match self.connections.get_mut(&fd) {
            Some(c) => c,
            None => return Ok(()),
        };

        // Write data
        conn.write()?;

        // Update poller based on new state
        match conn.state {
            ConnectionState::Reading => {
                self.poller.modify(fd, EventType::Read)?;
            }
            ConnectionState::Closed => {
                // Will be cleaned up in process_connections
            }
            _ => {}
        }

        Ok(())
    }

    /// Processes all connections (timeouts, pending requests)
    fn process_connections(&mut self) -> Result<()> {
        let mut to_remove = Vec::new();
        let mut to_process = Vec::new();

        // Find connections to process or remove
        for (&fd, conn) in &self.connections {
            // Check for timeout
            let timeout = self.config.servers.first()
                .map(|s| s.timeout)
                .unwrap_or(60);

            if conn.is_timed_out(timeout) {
                to_remove.push(fd);
                continue;
            }

            // Check for closed connections
            if conn.state == ConnectionState::Closed {
                to_remove.push(fd);
                continue;
            }

            // Check for requests ready to process
            if conn.state == ConnectionState::Processing {
                to_process.push(fd);
            }
        }

        // Process pending requests
        for fd in to_process {
            if let Some(conn) = self.connections.get_mut(&fd) {
                let response = match conn.parse_request() {
                    Ok(request) => {
                        // Find the right server config
                        let server_config = self.config.servers.iter()
                            .find(|s| s.ports.contains(&conn.server_port))
                            .or_else(|| self.config.servers.first());

                        if let Some(server) = server_config {
                            // Check body size
                            if !server.is_body_size_allowed(request.body.len()) {
                                Response::payload_too_large()
                                    .html("<h1>413 Payload Too Large</h1>")
                            } else {
                                self.handler.handle(&request, server)
                            }
                        } else {
                            Response::internal_error()
                                .html("<h1>500 Internal Server Error</h1>")
                        }
                    }
                    Err(_) => {
                        Response::bad_request()
                            .html("<h1>400 Bad Request</h1>")
                    }
                };

                conn.set_response(response);
                self.poller.modify(fd, EventType::Write)?;
            }
        }

        // Remove closed/timed out connections
        for fd in to_remove {
            self.poller.unregister(fd)?;
            self.connections.remove(&fd);
        }

        Ok(())
    }

    /// Stops the event loop
    pub fn stop(&mut self) {
        self.running = false;
    }
}
