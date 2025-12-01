use crate::http::{Response, StatusCode};

/// Handles HTTP redirects
pub struct Redirect;

impl Redirect {
    /// Creates a redirect response
    pub fn to(location: &str, permanent: bool) -> Response {
        let status = if permanent {
            StatusCode::MovedPermanently
        } else {
            StatusCode::Found
        };

        let mut response = Response::new(status);
        response.headers.set("Location", location);
        
        // Add a simple HTML body for clients that don't follow redirects
        let html = format!(
            "<!DOCTYPE html>\n\
            <html>\n\
            <head>\n\
            <title>Redirect</title>\n\
            <meta http-equiv=\"refresh\" content=\"0; url={}\">\n\
            </head>\n\
            <body>\n\
            <h1>{} {}</h1>\n\
            <p>Redirecting to <a href=\"{}\">{}</a></p>\n\
            </body>\n\
            </html>",
            location,
            status.code(),
            status.reason(),
            location,
            location
        );

        response.html(&html)
    }

    /// Creates a 301 Moved Permanently redirect
    pub fn permanent(location: &str) -> Response {
        Self::to(location, true)
    }

    /// Creates a 302 Found (temporary) redirect
    pub fn temporary(location: &str) -> Response {
        Self::to(location, false)
    }

    /// Creates a 307 Temporary Redirect (preserves method)
    pub fn temporary_preserve_method(location: &str) -> Response {
        let mut response = Response::new(StatusCode::TemporaryRedirect);
        response.headers.set("Location", location);
        
        let html = format!(
            "<!DOCTYPE html>\n\
            <html>\n\
            <head><title>Temporary Redirect</title></head>\n\
            <body>\n\
            <h1>307 Temporary Redirect</h1>\n\
            <p>Redirecting to <a href=\"{}\">{}</a></p>\n\
            </body>\n\
            </html>",
            location, location
        );

        response.html(&html)
    }

    /// Creates a 308 Permanent Redirect (preserves method)
    pub fn permanent_preserve_method(location: &str) -> Response {
        let mut response = Response::new(StatusCode::PermanentRedirect);
        response.headers.set("Location", location);
        
        let html = format!(
            "<!DOCTYPE html>\n\
            <html>\n\
            <head><title>Permanent Redirect</title></head>\n\
            <body>\n\
            <h1>308 Permanent Redirect</h1>\n\
            <p>Redirecting to <a href=\"{}\">{}</a></p>\n\
            </body>\n\
            </html>",
            location, location
        );

        response.html(&html)
    }
}
