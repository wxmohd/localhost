use std::fmt;

/// HTTP status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    // 2xx Success
    Ok = 200,
    Created = 201,
    NoContent = 204,

    // 3xx Redirection
    MovedPermanently = 301,
    Found = 302,
    NotModified = 304,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    // 4xx Client Errors
    BadRequest = 400,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    RequestTimeout = 408,
    PayloadTooLarge = 413,

    // 5xx Server Errors
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
}

impl StatusCode {
    /// Returns the numeric status code
    pub fn code(&self) -> u16 {
        *self as u16
    }

    /// Returns the reason phrase for the status code
    pub fn reason(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::Created => "Created",
            StatusCode::NoContent => "No Content",
            StatusCode::MovedPermanently => "Moved Permanently",
            StatusCode::Found => "Found",
            StatusCode::NotModified => "Not Modified",
            StatusCode::TemporaryRedirect => "Temporary Redirect",
            StatusCode::PermanentRedirect => "Permanent Redirect",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::RequestTimeout => "Request Timeout",
            StatusCode::PayloadTooLarge => "Payload Too Large",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::BadGateway => "Bad Gateway",
            StatusCode::ServiceUnavailable => "Service Unavailable",
        }
    }

    /// Creates a StatusCode from a numeric code
    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            200 => Some(StatusCode::Ok),
            201 => Some(StatusCode::Created),
            204 => Some(StatusCode::NoContent),
            301 => Some(StatusCode::MovedPermanently),
            302 => Some(StatusCode::Found),
            304 => Some(StatusCode::NotModified),
            307 => Some(StatusCode::TemporaryRedirect),
            308 => Some(StatusCode::PermanentRedirect),
            400 => Some(StatusCode::BadRequest),
            403 => Some(StatusCode::Forbidden),
            404 => Some(StatusCode::NotFound),
            405 => Some(StatusCode::MethodNotAllowed),
            408 => Some(StatusCode::RequestTimeout),
            413 => Some(StatusCode::PayloadTooLarge),
            500 => Some(StatusCode::InternalServerError),
            501 => Some(StatusCode::NotImplemented),
            502 => Some(StatusCode::BadGateway),
            503 => Some(StatusCode::ServiceUnavailable),
            _ => None,
        }
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.code(), self.reason())
    }
}
