pub mod pages;

use std::fmt;
use std::io;

/// Custom error types for the server
#[derive(Debug)]
pub enum ServerError {
    /// I/O errors (file, socket, etc.)
    Io(io::Error),
    /// Configuration parsing errors
    Config(String),
    /// HTTP parsing errors
    Parse(String),
    /// Request timeout
    Timeout,
    /// Client body too large
    BodyTooLarge,
    /// Method not allowed
    MethodNotAllowed,
    /// Resource not found
    NotFound,
    /// Permission denied
    Forbidden,
    /// Bad request
    BadRequest(String),
    /// Internal server error
    Internal(String),
    /// CGI execution error
    Cgi(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::Io(e) => write!(f, "I/O error: {}", e),
            ServerError::Config(msg) => write!(f, "Config error: {}", msg),
            ServerError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ServerError::Timeout => write!(f, "Request timeout"),
            ServerError::BodyTooLarge => write!(f, "Request body too large"),
            ServerError::MethodNotAllowed => write!(f, "Method not allowed"),
            ServerError::NotFound => write!(f, "Not found"),
            ServerError::Forbidden => write!(f, "Forbidden"),
            ServerError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ServerError::Internal(msg) => write!(f, "Internal error: {}", msg),
            ServerError::Cgi(msg) => write!(f, "CGI error: {}", msg),
        }
    }
}

impl std::error::Error for ServerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ServerError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ServerError {
    fn from(err: io::Error) -> Self {
        ServerError::Io(err)
    }
}

/// Result type alias for server operations
pub type Result<T> = std::result::Result<T, ServerError>;
