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
        
        // Remove route prefix from request path
        let relative = if self.path == "/" {
            request_path
        } else {
            request_path.strip_prefix(&self.path).unwrap_or(request_path)
        };

        // Clean up the path
        let relative = relative.trim_start_matches('/');
        
        if relative.is_empty() {
            // Return index file if path is directory
            if let Some(ref index) = self.index {
                Some(format!("{}/{}", root, index))
            } else {
                Some(root.clone())
            }
        } else {
            Some(format!("{}/{}", root, relative))
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
