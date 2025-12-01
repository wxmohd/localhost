use std::collections::HashMap;

/// Represents an HTTP cookie
#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub max_age: Option<u64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<SameSite>,
}

/// SameSite attribute values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl Cookie {
    /// Creates a new cookie with the given name and value
    pub fn new(name: &str, value: &str) -> Self {
        Cookie {
            name: name.to_string(),
            value: value.to_string(),
            path: Some("/".to_string()),
            domain: None,
            max_age: None,
            secure: false,
            http_only: true,
            same_site: Some(SameSite::Lax),
        }
    }

    /// Sets the path attribute
    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    /// Sets the domain attribute
    pub fn domain(mut self, domain: &str) -> Self {
        self.domain = Some(domain.to_string());
        self
    }

    /// Sets the max-age attribute (in seconds)
    pub fn max_age(mut self, seconds: u64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// Sets the secure attribute
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Sets the http-only attribute
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Sets the same-site attribute
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    /// Creates a session cookie (expires when browser closes)
    pub fn session(name: &str, value: &str) -> Self {
        Cookie::new(name, value)
    }

    /// Creates a persistent cookie with the given max-age
    pub fn persistent(name: &str, value: &str, max_age_seconds: u64) -> Self {
        Cookie::new(name, value).max_age(max_age_seconds)
    }

    /// Creates a cookie that will be deleted
    pub fn delete(name: &str) -> Self {
        Cookie::new(name, "").max_age(0)
    }

    /// Converts the cookie to a Set-Cookie header value
    pub fn to_header_value(&self) -> String {
        let mut parts = vec![format!("{}={}", self.name, self.value)];

        if let Some(ref path) = self.path {
            parts.push(format!("Path={}", path));
        }

        if let Some(ref domain) = self.domain {
            parts.push(format!("Domain={}", domain));
        }

        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age));
        }

        if self.secure {
            parts.push("Secure".to_string());
        }

        if self.http_only {
            parts.push("HttpOnly".to_string());
        }

        if let Some(same_site) = self.same_site {
            let value = match same_site {
                SameSite::Strict => "Strict",
                SameSite::Lax => "Lax",
                SameSite::None => "None",
            };
            parts.push(format!("SameSite={}", value));
        }

        parts.join("; ")
    }
}

/// Parses cookies from a Cookie header value
pub fn parse_cookies(header_value: &str) -> HashMap<String, String> {
    let mut cookies = HashMap::new();

    for part in header_value.split(';') {
        let part = part.trim();
        if let Some(pos) = part.find('=') {
            let name = part[..pos].trim().to_string();
            let value = part[pos + 1..].trim().to_string();
            cookies.insert(name, value);
        }
    }

    cookies
}
