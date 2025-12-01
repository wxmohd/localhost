use super::directory::DirectoryListing;
use super::redirect::Redirect;
use super::static_files::StaticFiles;
use crate::cgi::CgiExecutor;
use crate::config::{Config, ServerConfig};
use crate::http::{Method, Request, Response};
use std::fs;
use std::path::Path;

/// Main request handler
pub struct Handler {
    config: Config,
}

impl Handler {
    /// Creates a new handler with the given configuration
    pub fn new(config: Config) -> Self {
        Handler { config }
    }

    /// Handles an incoming request
    pub fn handle(&self, request: &Request, server: &ServerConfig) -> Response {
        // Find matching route
        let route = match server.find_route(&request.path) {
            Some(r) => r,
            None => return self.error_response(server, 404),
        };
        

        // Check if method is allowed
        if !route.is_method_allowed(&request.method) {
            return self.error_response(server, 405);
        }

        // Handle redirects
        if let Some((location, permanent)) = &route.redirect {
            return Redirect::to(location, *permanent);
        }

        // Resolve file path (use server root if route has no root)
        let file_path = match route.resolve_path_with_root(&request.path, &server.root) {
            Some(p) => p,
            None => return self.error_response(server, 404),
        };
        

        let path = Path::new(&file_path);

        // Check for CGI
        if let Some(cgi_handler) = route.get_cgi_handler(&file_path) {
            return self.handle_cgi(request, &file_path, cgi_handler, server);
        }

        // Handle based on method
        match request.method {
            Method::Get | Method::Head => {
                self.handle_get(request, &file_path, route.autoindex, route.index.as_deref(), server)
            }
            Method::Post => {
                self.handle_post(request, &file_path, route.upload_dir.as_deref(), server)
            }
            Method::Delete => {
                self.handle_delete(&file_path, server)
            }
            _ => self.error_response(server, 405),
        }
    }

    /// Handles GET requests
    fn handle_get(
        &self,
        request: &Request,
        file_path: &str,
        autoindex: bool,
        index: Option<&str>,
        server: &ServerConfig,
    ) -> Response {
        let path = Path::new(file_path);

        // If it's a directory
        if path.is_dir() {
            // Try index file first
            if let Some(index_file) = index {
                let index_path = path.join(index_file);
                if index_path.exists() && index_path.is_file() {
                    return match StaticFiles::serve(index_path.to_str().unwrap_or(file_path)) {
                        Ok(response) => response,
                        Err(_) => self.error_response(server, 500),
                    };
                }
            }

            // Directory listing if enabled
            if autoindex {
                return DirectoryListing::generate(file_path, &request.path);
            }

            return self.error_response(server, 403);
        }

        // Serve static file
        match StaticFiles::serve(file_path) {
            Ok(mut response) => {
                // For HEAD requests, remove body but keep headers
                if request.method == Method::Head {
                    response.body.clear();
                }
                response
            }
            Err(crate::error::ServerError::NotFound) => self.error_response(server, 404),
            Err(crate::error::ServerError::Forbidden) => self.error_response(server, 403),
            Err(_) => self.error_response(server, 500),
        }
    }

    /// Handles POST requests (file uploads)
    fn handle_post(
        &self,
        request: &Request,
        file_path: &str,
        upload_dir: Option<&str>,
        server: &ServerConfig,
    ) -> Response {
        // Check for upload directory
        let upload_path = match upload_dir {
            Some(dir) => dir,
            None => return self.error_response(server, 403),
        };

        // Create upload directory if it doesn't exist
        if let Err(_) = fs::create_dir_all(upload_path) {
            return self.error_response(server, 500);
        }

        // Parse multipart form data or save raw body
        let content_type = request.content_type().unwrap_or("");
        
        if content_type.starts_with("multipart/form-data") {
            self.handle_multipart_upload(request, upload_path, server)
        } else {
            // Save raw body as file
            let filename = format!("upload_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0));
            
            let target_path = Path::new(upload_path).join(&filename);
            
            match fs::write(&target_path, &request.body) {
                Ok(_) => Response::ok()
                    .json(&format!("{{\"status\":\"ok\",\"file\":\"{}\"}}", filename)),
                Err(_) => self.error_response(server, 500),
            }
        }
    }

    /// Handles multipart form data uploads
    fn handle_multipart_upload(
        &self,
        request: &Request,
        upload_path: &str,
        server: &ServerConfig,
    ) -> Response {
        let content_type = request.content_type().unwrap_or("");
        
        // Extract boundary
        let boundary = content_type
            .split("boundary=")
            .nth(1)
            .map(|b| b.trim_matches('"'));

        let boundary = match boundary {
            Some(b) => format!("--{}", b),
            None => return self.error_response(server, 400),
        };

        // Parse multipart data (simplified)
        let body = match std::str::from_utf8(&request.body) {
            Ok(s) => s,
            Err(_) => return self.error_response(server, 400),
        };

        let mut uploaded_files = Vec::new();

        for part in body.split(&boundary) {
            if part.trim().is_empty() || part.trim() == "--" {
                continue;
            }

            // Find filename in Content-Disposition header
            if let Some(filename) = Self::extract_filename(part) {
                // Find content (after double newline)
                if let Some(content_start) = part.find("\r\n\r\n") {
                    let content = &part[content_start + 4..];
                    let content = content.trim_end_matches("\r\n");
                    
                    let target_path = Path::new(upload_path).join(&filename);
                    
                    if fs::write(&target_path, content.as_bytes()).is_ok() {
                        uploaded_files.push(filename);
                    }
                }
            }
        }

        if uploaded_files.is_empty() {
            self.error_response(server, 400)
        } else {
            Response::ok()
                .json(&format!("{{\"status\":\"ok\",\"files\":{:?}}}", uploaded_files))
        }
    }

    /// Extracts filename from multipart part
    fn extract_filename(part: &str) -> Option<String> {
        for line in part.lines() {
            if line.to_lowercase().contains("content-disposition") {
                if let Some(start) = line.find("filename=\"") {
                    let rest = &line[start + 10..];
                    if let Some(end) = rest.find('"') {
                        return Some(rest[..end].to_string());
                    }
                }
            }
        }
        None
    }

    /// Handles DELETE requests
    fn handle_delete(&self, file_path: &str, server: &ServerConfig) -> Response {
        let path = Path::new(file_path);

        if !path.exists() {
            return self.error_response(server, 404);
        }

        // Security: don't allow deleting directories
        if path.is_dir() {
            return self.error_response(server, 403);
        }

        match fs::remove_file(path) {
            Ok(_) => Response::ok()
                .json("{\"status\":\"ok\",\"message\":\"File deleted\"}"),
            Err(_) => self.error_response(server, 500),
        }
    }

    /// Handles CGI requests
    fn handle_cgi(
        &self,
        request: &Request,
        script_path: &str,
        interpreter: &str,
        server: &ServerConfig,
    ) -> Response {
        match CgiExecutor::execute(request, script_path, interpreter) {
            Ok(response) => response,
            Err(_) => self.error_response(server, 500),
        }
    }

    /// Generates an error response
    fn error_response(&self, server: &ServerConfig, status_code: u16) -> Response {
        use crate::http::StatusCode;
        
        // Try custom error page
        if let Some(error_page) = server.get_error_page(status_code) {
            let error_path = format!("{}/{}", server.root, error_page.trim_start_matches('/'));
            if let Ok(mut response) = StaticFiles::serve(&error_path) {
                // Set correct status code for error page
                response.status = StatusCode::from_code(status_code).unwrap_or(StatusCode::InternalServerError);
                return response;
            }
        }

        // Default error pages
        let (response, message) = match status_code {
            400 => (Response::bad_request(), "Bad Request"),
            403 => (Response::forbidden(), "Forbidden"),
            404 => (Response::not_found(), "Not Found"),
            405 => (Response::method_not_allowed(), "Method Not Allowed"),
            413 => (Response::payload_too_large(), "Payload Too Large"),
            _ => (Response::internal_error(), "Internal Server Error"),
        };

        response.html(&format!(
            "<!DOCTYPE html>\n\
            <html>\n\
            <head><title>{} {}</title></head>\n\
            <body>\n\
            <h1>{} {}</h1>\n\
            </body>\n\
            </html>",
            status_code, message, status_code, message
        ))
    }
}
