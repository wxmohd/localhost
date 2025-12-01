pub mod parser;
pub mod route;
pub mod server_config;

pub use parser::ConfigParser;
pub use route::Route;
pub use server_config::{Config, ServerConfig};

impl Config {
    /// Loads configuration from a file path
    pub fn load(path: &str) -> crate::error::Result<Self> {
        ConfigParser::parse(path)
    }
}
