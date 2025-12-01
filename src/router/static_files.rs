use crate::error::{Result, ServerError};
use crate::http::{mime_type, Response};
use std::fs;
use std::path::{Path, PathBuf};

/// Serves static files from the filesystem
pub struct StaticFiles;

impl StaticFiles {
    /// Serves a file from the given path
    pub fn serve(file_path: &str) -> Result<Response> {
        let path = Path::new(file_path);

        // Security: prevent path traversal
        if !Self::is_safe_path(file_path) {
            return Err(ServerError::Forbidden);
        }

        // Check if file exists
        if !path.exists() {
            return Err(ServerError::NotFound);
        }

        // Check if it's a file (not directory)
        if !path.is_file() {
            return Err(ServerError::NotFound);
        }

        // Read file contents
        let contents = fs::read(path)?;
        let mime = mime_type(file_path);

        Ok(Response::ok()
            .content_type(mime)
            .body(contents))
    }

    /// Serves a file with a fallback to index file for directories
    pub fn serve_with_index(file_path: &str, index: &str) -> Result<Response> {
        let path = Path::new(file_path);

        if path.is_dir() {
            let index_path = path.join(index);
            if index_path.exists() && index_path.is_file() {
                return Self::serve(index_path.to_str().unwrap_or(file_path));
            }
            return Err(ServerError::NotFound);
        }

        Self::serve(file_path)
    }

    /// Checks if a path is safe (no path traversal)
    fn is_safe_path(path: &str) -> bool {
        // Reject paths with ..
        if path.contains("..") {
            return false;
        }

        // Reject paths starting with /
        // (they should be resolved relative to root)
        let normalized = path.replace('\\', "/");
        
        // Check for null bytes
        if path.contains('\0') {
            return false;
        }

        true
    }

    /// Resolves a request path to a filesystem path
    pub fn resolve_path(root: &str, request_path: &str) -> PathBuf {
        let root = Path::new(root);
        let clean_path = request_path.trim_start_matches('/');
        root.join(clean_path)
    }

    /// Gets file metadata
    pub fn file_info(path: &str) -> Option<FileInfo> {
        let metadata = fs::metadata(path).ok()?;
        
        Some(FileInfo {
            size: metadata.len(),
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
        })
    }
}

/// File information
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub size: u64,
    pub is_dir: bool,
    pub is_file: bool,
}
