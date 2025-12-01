use super::headers::Headers;
use super::method::Method;
use std::collections::HashMap;

/// Represents an HTTP request
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method (GET, POST, DELETE, etc.)
    pub method: Method,
    /// Request URI path
    pub path: String,
    /// Query string parameters
    pub query: HashMap<String, String>,
    /// HTTP version (e.g., "HTTP/1.1")
    pub version: String,
    /// Request headers
    pub headers: Headers,
    /// Request body
    pub body: Vec<u8>,
}

impl Request {
    /// Creates a new Request with the given method and path
    pub fn new(method: Method, path: &str) -> Self {
        let (path, query) = Self::parse_path_and_query(path);
        Request {
            method,
            path,
            query,
            version: "HTTP/1.1".to_string(),
            headers: Headers::new(),
            body: Vec::new(),
        }
    }

    /// Parses path and query string from URI
    fn parse_path_and_query(uri: &str) -> (String, HashMap<String, String>) {
        let mut query = HashMap::new();
        
        if let Some(pos) = uri.find('?') {
            let path = uri[..pos].to_string();
            let query_str = &uri[pos + 1..];
            
            for pair in query_str.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = Self::url_decode(&pair[..eq_pos]);
                    let value = Self::url_decode(&pair[eq_pos + 1..]);
                    query.insert(key, value);
                } else if !pair.is_empty() {
                    query.insert(Self::url_decode(pair), String::new());
                }
            }
            
            (path, query)
        } else {
            (uri.to_string(), query)
        }
    }

    /// URL decodes a string
    fn url_decode(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                        continue;
                    }
                }
                result.push('%');
                result.push_str(&hex);
            } else if c == '+' {
                result.push(' ');
            } else {
                result.push(c);
            }
        }
        
        result
    }

    /// Returns the Host header value
    pub fn host(&self) -> Option<&str> {
        self.headers.host()
    }

    /// Returns the Content-Length
    pub fn content_length(&self) -> Option<usize> {
        self.headers.content_length()
    }

    /// Returns the Content-Type
    pub fn content_type(&self) -> Option<&str> {
        self.headers.content_type()
    }

    /// Checks if the request uses chunked transfer encoding
    pub fn is_chunked(&self) -> bool {
        self.headers.is_chunked()
    }

    /// Checks if connection should be kept alive
    pub fn keep_alive(&self) -> bool {
        self.headers.keep_alive()
    }

    /// Gets a query parameter by name
    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query.get(name).map(|s| s.as_str())
    }

    /// Gets a cookie value by name
    pub fn cookie(&self, name: &str) -> Option<String> {
        self.headers.get("cookie").and_then(|cookies| {
            for cookie in cookies.split(';') {
                let cookie = cookie.trim();
                if let Some(pos) = cookie.find('=') {
                    if &cookie[..pos] == name {
                        return Some(cookie[pos + 1..].to_string());
                    }
                }
            }
            None
        })
    }

    /// Returns the body as a string (if valid UTF-8)
    pub fn body_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }
}
