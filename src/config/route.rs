use crate::http::Method;
use std::collections::HashMap;

/// Route configuration for a location block
#[derive(Debug, Clone)]
pub struct Route {
    /// URL path pattern (e.g., "/", "/api", "/images")
    pub path: String,
    /// Allowed HTTP methods
    pub methods: Vec<Method>,
    /// Root directory for serving files
    pub root: Option<String>,
    /// Default index file (e.g., "index.html")
    pub index: Option<String>,
    /// Enable directory listing
    pub autoindex: bool,
    /// HTTP redirect (target URL, permanent flag)
    pub redirect: Option<(String, bool)>,
    /// CGI handlers by file extension (e.g., ".py" -> "/usr/bin/python3")
    pub cgi: HashMap<String, String>,
    /// Upload directory for file uploads
    pub upload_dir: Option<String>,
}

impl Route {
    /// Creates a new Route with default settings
    pub fn new(path: &str) -> Self {
        Route {
            path: path.to_string(),
            methods: vec![Method::Get],
            root: None,
            index: Some("index.html".to_string()),
            autoindex: false,
            redirect: None,
            cgi: HashMap::new(),
            upload_dir: None,
        }
    }

    /// Checks if a method is allowed for this route
    pub fn is_method_allowed(&self, method: &Method) -> bool {
        self.methods.contains(method)
    }

    /// Checks if the given path matches this route
    pub fn matches(&self, request_path: &str) -> bool {
        if self.path == "/" {
            return true;
        }
        request_path.starts_with(&self.path)
    }

    /// Returns the file path for a request, resolving against root
    pub fn resolve_path(&self, request_path: &str) -> Option<String> {
        let root = self.root.as_ref()?;
        let relative = if self.path == "/" {
            request_path
        } else {
            request_path.strip_prefix(&self.path).unwrap_or(request_path)
        };
        let relative = relative.trim_start_matches('/');
        
        if relative.is_empty() {
            Some(root.clone())
        } else {
            Some(format!("{}/{}", root, relative))
        }
    }

    /// Returns the file path using provided root as fallback
    pub fn resolve_path_with_root(&self, request_path: &str, server_root: &str) -> Option<String> {
        // If route has custom root, use it and strip route prefix
        // Otherwise use server root and keep full request path
        if let Some(ref custom_root) = self.root {
            // Route has its own root - strip route prefix
            let relative = if self.path == "/" {
                request_path
            } else {
                request_path.strip_prefix(&self.path).unwrap_or(request_path)
            };
            let relative = relative.trim_start_matches('/');
            
            if relative.is_empty() {
                Some(custom_root.clone())
            } else {
                Some(format!("{}/{}", custom_root, relative))
            }
        } else {
            // Using server root - map request path directly to server root
            let clean_path = request_path.trim_start_matches('/');
            if clean_path.is_empty() {
                Some(server_root.to_string())
            } else {
                Some(format!("{}/{}", server_root, clean_path))
            }
        }
    }

    /// Gets the CGI handler for a file extension
    pub fn get_cgi_handler(&self, path: &str) -> Option<&String> {
        for (ext, handler) in &self.cgi {
            if path.ends_with(ext) {
                return Some(handler);
            }
        }
        None
    }

    /// Checks if this route has a redirect
    pub fn has_redirect(&self) -> bool {
        self.redirect.is_some()
    }
}

impl Default for Route {
    fn default() -> Self {
        Self::new("/")
    }
}
