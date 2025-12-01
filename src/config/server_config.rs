use super::route::Route;
use std::collections::HashMap;

/// Configuration for a single virtual server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server name (for virtual hosting)
    pub server_name: String,
    /// Host address to bind to
    pub host: String,
    /// Ports to listen on
    pub ports: Vec<u16>,
    /// Root directory for serving files
    pub root: String,
    /// Maximum client body size in bytes
    pub client_max_body_size: usize,
    /// Custom error pages (status code -> file path)
    pub error_pages: HashMap<u16, String>,
    /// Route configurations
    pub routes: Vec<Route>,
    /// Request timeout in seconds
    pub timeout: u64,
}

impl ServerConfig {
    /// Creates a new ServerConfig with default settings
    pub fn new() -> Self {
        ServerConfig {
            server_name: "localhost".to_string(),
            host: "127.0.0.1".to_string(),
            ports: vec![8080],
            root: "./www".to_string(),
            client_max_body_size: 10 * 1024 * 1024, // 10MB
            error_pages: HashMap::new(),
            routes: vec![Route::default()],
            timeout: 60,
        }
    }

    /// Finds the best matching route for a request path
    pub fn find_route(&self, path: &str) -> Option<&Route> {
        // Find the most specific matching route (longest path)
        self.routes
            .iter()
            .filter(|r| r.matches(path))
            .max_by_key(|r| r.path.len())
    }

    /// Gets the error page path for a status code
    pub fn get_error_page(&self, status_code: u16) -> Option<&String> {
        self.error_pages.get(&status_code)
    }

    /// Checks if the body size is within limits
    pub fn is_body_size_allowed(&self, size: usize) -> bool {
        size <= self.client_max_body_size
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Global configuration containing all servers
#[derive(Debug, Clone)]
pub struct Config {
    /// All server configurations
    pub servers: Vec<ServerConfig>,
}

impl Config {
    /// Creates a new empty Config
    pub fn new() -> Self {
        Config {
            servers: Vec::new(),
        }
    }

    /// Validates the configuration and returns an error if invalid
    pub fn validate(&self) -> Result<(), String> {
        if self.servers.is_empty() {
            return Err("No servers configured".to_string());
        }

        // Check for duplicate port bindings on same host
        let mut seen_bindings: Vec<(String, u16)> = Vec::new();
        for server in &self.servers {
            for port in &server.ports {
                let binding = (server.host.clone(), *port);
                // Allow same port if different server_name (virtual hosting)
                let duplicate = seen_bindings.iter().any(|(h, p)| {
                    h == &server.host && p == port
                });
                
                // Check if there's already a server with same host:port but different server_name
                let virtual_host_exists = self.servers.iter().any(|s| {
                    s.host == server.host && 
                    s.ports.contains(port) && 
                    s.server_name != server.server_name
                });
                
                if duplicate && !virtual_host_exists {
                    return Err(format!(
                        "Duplicate port binding: {}:{} (use different server_name for virtual hosting)",
                        server.host, port
                    ));
                }
                seen_bindings.push(binding);
            }
        }

        // Validate each server
        for server in &self.servers {
            if server.ports.is_empty() {
                return Err(format!("Server '{}' has no ports configured", server.server_name));
            }
            if server.root.is_empty() {
                return Err(format!("Server '{}' has no root directory", server.server_name));
            }
        }

        Ok(())
    }

    /// Finds the server config for a given host:port and server_name
    pub fn find_server(&self, host: &str, port: u16, server_name: Option<&str>) -> Option<&ServerConfig> {
        // First try to find exact match with server_name
        if let Some(name) = server_name {
            if let Some(server) = self.servers.iter().find(|s| {
                s.ports.contains(&port) && s.server_name == name
            }) {
                return Some(server);
            }
        }

        // Fall back to first server matching port (default server)
        self.servers
            .iter()
            .find(|s| s.ports.contains(&port))
    }

    /// Finds server by Host header (for virtual hosting)
    pub fn find_server_by_host(&self, host_header: &str, port: u16) -> Option<&ServerConfig> {
        // Extract hostname without port
        let hostname = host_header.split(':').next().unwrap_or(host_header);
        
        // Try exact server_name match first
        if let Some(server) = self.servers.iter().find(|s| {
            s.ports.contains(&port) && s.server_name == hostname
        }) {
            return Some(server);
        }

        // Fall back to first server on this port (default)
        self.servers
            .iter()
            .find(|s| s.ports.contains(&port))
    }

    /// Gets all unique host:port combinations
    pub fn get_listen_addresses(&self) -> Vec<(String, u16)> {
        use std::collections::HashSet;
        let mut seen: HashSet<(String, u16)> = HashSet::new();
        let mut addresses: Vec<(String, u16)> = Vec::new();
        
        for server in &self.servers {
            for &port in &server.ports {
                let key = (server.host.clone(), port);
                if seen.insert(key.clone()) {
                    addresses.push(key);
                }
            }
        }
        addresses
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Self::new();
        config.servers.push(ServerConfig::default());
        config
    }
}
