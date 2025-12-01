use super::headers::Headers;
use super::status::StatusCode;

/// Represents an HTTP response
#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP version
    pub version: String,
    /// Status code
    pub status: StatusCode,
    /// Response headers
    pub headers: Headers,
    /// Response body
    pub body: Vec<u8>,
}

impl Response {
    /// Creates a new Response with the given status code
    pub fn new(status: StatusCode) -> Self {
        let mut headers = Headers::new();
        headers.set("Server", "localhost/0.1.0");
        
        Response {
            version: "HTTP/1.1".to_string(),
            status,
            headers,
            body: Vec::new(),
        }
    }

    /// Creates a 200 OK response
    pub fn ok() -> Self {
        Self::new(StatusCode::Ok)
    }

    /// Creates a 404 Not Found response
    pub fn not_found() -> Self {
        Self::new(StatusCode::NotFound)
    }

    /// Creates a 400 Bad Request response
    pub fn bad_request() -> Self {
        Self::new(StatusCode::BadRequest)
    }

    /// Creates a 403 Forbidden response
    pub fn forbidden() -> Self {
        Self::new(StatusCode::Forbidden)
    }

    /// Creates a 405 Method Not Allowed response
    pub fn method_not_allowed() -> Self {
        Self::new(StatusCode::MethodNotAllowed)
    }

    /// Creates a 413 Payload Too Large response
    pub fn payload_too_large() -> Self {
        Self::new(StatusCode::PayloadTooLarge)
    }

    /// Creates a 500 Internal Server Error response
    pub fn internal_error() -> Self {
        Self::new(StatusCode::InternalServerError)
    }

    /// Creates a redirect response
    pub fn redirect(location: &str, permanent: bool) -> Self {
        let status = if permanent {
            StatusCode::MovedPermanently
        } else {
            StatusCode::Found
        };
        let mut response = Self::new(status);
        response.headers.set("Location", location);
        response
    }

    /// Sets the Content-Type header
    pub fn content_type(mut self, mime_type: &str) -> Self {
        self.headers.set("Content-Type", mime_type);
        self
    }

    /// Sets the body from bytes
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.headers.set("Content-Length", &body.len().to_string());
        self.body = body;
        self
    }

    /// Sets the body from a string
    pub fn body_str(self, body: &str) -> Self {
        self.body(body.as_bytes().to_vec())
    }

    /// Sets an HTML body
    pub fn html(self, html: &str) -> Self {
        self.content_type("text/html; charset=utf-8")
            .body_str(html)
    }

    /// Sets a JSON body
    pub fn json(self, json: &str) -> Self {
        self.content_type("application/json")
            .body_str(json)
    }

    /// Sets a plain text body
    pub fn text(self, text: &str) -> Self {
        self.content_type("text/plain; charset=utf-8")
            .body_str(text)
    }

    /// Adds a Set-Cookie header
    pub fn set_cookie(mut self, name: &str, value: &str, options: Option<&str>) -> Self {
        let cookie = match options {
            Some(opts) => format!("{}={}; {}", name, value, opts),
            None => format!("{}={}", name, value),
        };
        self.headers.add("Set-Cookie", &cookie);
        self
    }

    /// Sets the Connection header
    pub fn connection(mut self, value: &str) -> Self {
        self.headers.set("Connection", value);
        self
    }

    /// Serializes the response to bytes for sending
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Status line
        let status_line = format!(
            "{} {}\r\n",
            self.version,
            self.status
        );
        result.extend_from_slice(status_line.as_bytes());
        
        // Headers
        result.extend_from_slice(self.headers.to_http_string().as_bytes());
        
        // Empty line separating headers from body
        result.extend_from_slice(b"\r\n");
        
        // Body
        result.extend_from_slice(&self.body);
        
        result
    }
}

/// Helper to determine MIME type from file extension
pub fn mime_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext.to_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}
