pub mod connection;
pub mod epoll;
pub mod event_loop;
pub mod listener;

pub use connection::{Connection, ConnectionState};
pub use epoll::{Event, EventType, Poller};
pub use event_loop::EventLoop;
pub use listener::Listener;

use crate::config::Config;
use crate::error::Result;

/// Main server struct
pub struct Server;

impl Server {
    /// Runs the server with the given configuration
    pub fn run(config: Config) -> Result<()> {
        let mut event_loop = EventLoop::new(config)?;
        event_loop.run()
    }
}
