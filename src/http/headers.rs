use std::collections::HashMap;

/// HTTP headers collection (case-insensitive keys)
#[derive(Debug, Clone, Default)]
pub struct Headers {
    inner: HashMap<String, Vec<String>>,
}

impl Headers {
    /// Creates a new empty Headers collection
    pub fn new() -> Self {
        Headers {
            inner: HashMap::new(),
        }
    }

    /// Gets the first value for a header (case-insensitive)
    pub fn get(&self, name: &str) -> Option<&str> {
        self.inner
            .get(&name.to_lowercase())
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }

    /// Gets all values for a header (case-insensitive)
    pub fn get_all(&self, name: &str) -> Option<&Vec<String>> {
        self.inner.get(&name.to_lowercase())
    }

    /// Sets a header value, replacing any existing values
    pub fn set(&mut self, name: &str, value: &str) {
        self.inner
            .insert(name.to_lowercase(), vec![value.to_string()]);
    }

    /// Adds a header value, preserving existing values
    pub fn add(&mut self, name: &str, value: &str) {
        self.inner
            .entry(name.to_lowercase())
            .or_insert_with(Vec::new)
            .push(value.to_string());
    }

    /// Removes a header
    pub fn remove(&mut self, name: &str) {
        self.inner.remove(&name.to_lowercase());
    }

    /// Checks if a header exists
    pub fn contains(&self, name: &str) -> bool {
        self.inner.contains_key(&name.to_lowercase())
    }

    /// Returns the Content-Length header value if present
    pub fn content_length(&self) -> Option<usize> {
        self.get("content-length")
            .and_then(|v| v.parse().ok())
    }

    /// Returns the Content-Type header value if present
    pub fn content_type(&self) -> Option<&str> {
        self.get("content-type")
    }

    /// Returns the Host header value if present
    pub fn host(&self) -> Option<&str> {
        self.get("host")
    }

    /// Returns the Connection header value if present
    pub fn connection(&self) -> Option<&str> {
        self.get("connection")
    }

    /// Checks if connection should be kept alive
    pub fn keep_alive(&self) -> bool {
        self.connection()
            .map(|c| c.to_lowercase() != "close")
            .unwrap_or(true)
    }

    /// Returns the Transfer-Encoding header value if present
    pub fn transfer_encoding(&self) -> Option<&str> {
        self.get("transfer-encoding")
    }

    /// Checks if transfer encoding is chunked
    pub fn is_chunked(&self) -> bool {
        self.transfer_encoding()
            .map(|te| te.to_lowercase().contains("chunked"))
            .unwrap_or(false)
    }

    /// Returns an iterator over all headers
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.inner.iter()
    }

    /// Serializes headers to HTTP format
    pub fn to_http_string(&self) -> String {
        let mut result = String::new();
        for (name, values) in &self.inner {
            for value in values {
                result.push_str(&format!("{}: {}\r\n", name, value));
            }
        }
        result
    }
}
