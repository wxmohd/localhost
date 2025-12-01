use super::route::Route;
use super::server_config::{Config, ServerConfig};
use crate::error::{Result, ServerError};
use crate::http::Method;
use std::fs;

/// Configuration file parser
pub struct ConfigParser;

impl ConfigParser {
    /// Loads and parses a configuration file
    pub fn parse(path: &str) -> Result<Config> {
        let content = fs::read_to_string(path)
            .map_err(|e| ServerError::Config(format!("Failed to read config file: {}", e)))?;

        Self::parse_string(&content)
    }

    /// Parses configuration from a string
    pub fn parse_string(content: &str) -> Result<Config> {
        let mut config = Config::new();
        let mut chars = content.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                continue;
            }

            // Read directive name
            let mut directive = String::new();
            directive.push(c);
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() || ch == '{' {
                    break;
                }
                directive.push(chars.next().unwrap());
            }

            if directive == "server" {
                // Skip whitespace and opening brace
                Self::skip_whitespace(&mut chars);
                if chars.next() != Some('{') {
                    return Err(ServerError::Config("Expected '{' after server".to_string()));
                }

                let server = Self::parse_server_block(&mut chars)?;
                config.servers.push(server);
            }
        }

        if config.servers.is_empty() {
            config.servers.push(ServerConfig::default());
        }

        Ok(config)
    }

    /// Parses a server block
    fn parse_server_block(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<ServerConfig> {
        let mut server = ServerConfig::new();
        server.routes.clear(); // Clear default route

        loop {
            Self::skip_whitespace(chars);

            match chars.peek() {
                Some('}') => {
                    chars.next();
                    break;
                }
                Some(_) => {
                    let directive = Self::read_word(chars);
                    Self::skip_whitespace(chars);

                    match directive.as_str() {
                        "listen" => {
                            let value = Self::read_value(chars);
                            if let Ok(port) = value.parse::<u16>() {
                                if !server.ports.contains(&port) {
                                    server.ports.push(port);
                                }
                            }
                        }
                        "server_name" => {
                            server.server_name = Self::read_value(chars);
                        }
                        "host" => {
                            server.host = Self::read_value(chars);
                        }
                        "root" => {
                            server.root = Self::read_value(chars);
                        }
                        "client_max_body_size" => {
                            let value = Self::read_value(chars);
                            server.client_max_body_size = Self::parse_size(&value)?;
                        }
                        "timeout" => {
                            let value = Self::read_value(chars);
                            server.timeout = value.parse().unwrap_or(60);
                        }
                        "error_page" => {
                            let code_str = Self::read_word(chars);
                            Self::skip_whitespace(chars);
                            let path = Self::read_value(chars);
                            if let Ok(code) = code_str.parse::<u16>() {
                                server.error_pages.insert(code, path);
                            }
                        }
                        "location" => {
                            let path = Self::read_word(chars);
                            Self::skip_whitespace(chars);
                            if chars.next() != Some('{') {
                                return Err(ServerError::Config("Expected '{' after location path".to_string()));
                            }
                            let mut route = Self::parse_location_block(chars, &path)?;
                            // Inherit root from server if not set
                            if route.root.is_none() {
                                route.root = Some(server.root.clone());
                            }
                            server.routes.push(route);
                        }
                        "" => continue,
                        _ => {
                            // Skip unknown directive
                            Self::read_value(chars);
                        }
                    }
                }
                None => break,
            }
        }

        // Add default route if none specified
        if server.routes.is_empty() {
            let mut default_route = Route::new("/");
            default_route.root = Some(server.root.clone());
            server.routes.push(default_route);
        }

        // Clear default port if others were specified
        if server.ports.len() > 1 && server.ports.contains(&8080) {
            server.ports.retain(|&p| p != 8080);
        }

        Ok(server)
    }

    /// Parses a location block
    fn parse_location_block(chars: &mut std::iter::Peekable<std::str::Chars>, path: &str) -> Result<Route> {
        let mut route = Route::new(path);
        route.methods.clear(); // Clear default methods

        loop {
            Self::skip_whitespace(chars);

            match chars.peek() {
                Some('}') => {
                    chars.next();
                    break;
                }
                Some(_) => {
                    let directive = Self::read_word(chars);
                    Self::skip_whitespace(chars);

                    match directive.as_str() {
                        "methods" | "allow_methods" => {
                            let methods_str = Self::read_value(chars);
                            for method in methods_str.split_whitespace() {
                                if let Ok(m) = method.parse::<Method>() {
                                    if !route.methods.contains(&m) {
                                        route.methods.push(m);
                                    }
                                }
                            }
                        }
                        "root" => {
                            route.root = Some(Self::read_value(chars));
                        }
                        "index" => {
                            route.index = Some(Self::read_value(chars));
                        }
                        "autoindex" => {
                            let value = Self::read_value(chars);
                            route.autoindex = value == "on" || value == "true";
                        }
                        "return" | "redirect" => {
                            let target = Self::read_value(chars);
                            let permanent = directive == "redirect";
                            route.redirect = Some((target, permanent));
                        }
                        "cgi" => {
                            let ext = Self::read_word(chars);
                            Self::skip_whitespace(chars);
                            let handler = Self::read_value(chars);
                            route.cgi.insert(ext, handler);
                        }
                        "upload_dir" => {
                            route.upload_dir = Some(Self::read_value(chars));
                        }
                        "" => continue,
                        _ => {
                            // Skip unknown directive
                            Self::read_value(chars);
                        }
                    }
                }
                None => break,
            }
        }

        // Default to GET if no methods specified
        if route.methods.is_empty() {
            route.methods.push(Method::Get);
        }

        Ok(route)
    }

    /// Skips whitespace characters
    fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
    }

    /// Reads a word (until whitespace or special char)
    fn read_word(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut word = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() || c == '{' || c == '}' || c == ';' {
                break;
            }
            word.push(chars.next().unwrap());
        }
        word
    }

    /// Reads a value (until newline or semicolon)
    fn read_value(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut value = String::new();
        while let Some(&c) = chars.peek() {
            if c == '\n' || c == '\r' || c == ';' || c == '{' || c == '}' {
                if c == ';' {
                    chars.next();
                }
                break;
            }
            value.push(chars.next().unwrap());
        }
        value.trim().to_string()
    }

    /// Parses a size string (e.g., "10M", "1024K", "1G")
    fn parse_size(s: &str) -> Result<usize> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(0);
        }

        let (num_str, multiplier) = if s.ends_with('K') || s.ends_with('k') {
            (&s[..s.len() - 1], 1024)
        } else if s.ends_with('M') || s.ends_with('m') {
            (&s[..s.len() - 1], 1024 * 1024)
        } else if s.ends_with('G') || s.ends_with('g') {
            (&s[..s.len() - 1], 1024 * 1024 * 1024)
        } else {
            (s, 1)
        };

        num_str
            .parse::<usize>()
            .map(|n| n * multiplier)
            .map_err(|_| ServerError::Config(format!("Invalid size: {}", s)))
    }
}
