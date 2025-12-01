use crate::http::Response;
use std::fs;
use std::path::Path;

/// Generates directory listings
pub struct DirectoryListing;

impl DirectoryListing {
    /// Generates an HTML directory listing for the given path
    pub fn generate(dir_path: &str, request_path: &str) -> Response {
        let path = Path::new(dir_path);

        if !path.is_dir() {
            return Response::not_found()
                .html("<h1>404 Not Found</h1>");
        }

        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => {
                return Response::forbidden()
                    .html("<h1>403 Forbidden</h1>");
            }
        };

        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"utf-8\">\n");
        html.push_str(&format!("<title>Index of {}</title>\n", request_path));
        html.push_str("<style>\n");
        html.push_str("body { font-family: sans-serif; margin: 20px; }\n");
        html.push_str("h1 { border-bottom: 1px solid #ccc; padding-bottom: 10px; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("th, td { text-align: left; padding: 8px; border-bottom: 1px solid #ddd; }\n");
        html.push_str("tr:hover { background-color: #f5f5f5; }\n");
        html.push_str("a { text-decoration: none; color: #0066cc; }\n");
        html.push_str("a:hover { text-decoration: underline; }\n");
        html.push_str(".dir { font-weight: bold; }\n");
        html.push_str(".size { color: #666; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        html.push_str(&format!("<h1>Index of {}</h1>\n", request_path));
        html.push_str("<table>\n");
        html.push_str("<tr><th>Name</th><th>Size</th><th>Type</th></tr>\n");

        // Parent directory link
        if request_path != "/" {
            let parent = Self::parent_path(request_path);
            html.push_str(&format!(
                "<tr><td class=\"dir\"><a href=\"{}\">..</a></td><td>-</td><td>Directory</td></tr>\n",
                parent
            ));
        }

        // Collect and sort entries
        let mut items: Vec<_> = entries
            .filter_map(|e| e.ok())
            .collect();
        
        items.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();
            
            // Directories first, then alphabetically
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        // Generate entries
        for entry in items {
            let name = entry.file_name().to_string_lossy().to_string();
            let entry_path = entry.path();
            let is_dir = entry_path.is_dir();
            
            let href = if request_path.ends_with('/') {
                format!("{}{}", request_path, name)
            } else {
                format!("{}/{}", request_path, name)
            };

            let (size_str, type_str) = if is_dir {
                ("-".to_string(), "Directory")
            } else {
                let size = entry.metadata()
                    .map(|m| Self::format_size(m.len()))
                    .unwrap_or_else(|_| "-".to_string());
                (size, "File")
            };

            let class = if is_dir { " class=\"dir\"" } else { "" };
            let display_name = if is_dir {
                format!("{}/", name)
            } else {
                name
            };

            html.push_str(&format!(
                "<tr><td{}><a href=\"{}\">{}</a></td><td class=\"size\">{}</td><td>{}</td></tr>\n",
                class, href, display_name, size_str, type_str
            ));
        }

        html.push_str("</table>\n");
        html.push_str("<hr>\n");
        html.push_str("<p><em>localhost server</em></p>\n");
        html.push_str("</body>\n</html>");

        Response::ok().html(&html)
    }

    /// Formats a file size in human-readable format
    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.1} GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1} MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1} KB", size as f64 / KB as f64)
        } else {
            format!("{} B", size)
        }
    }

    /// Gets the parent path
    fn parent_path(path: &str) -> String {
        let trimmed = path.trim_end_matches('/');
        match trimmed.rfind('/') {
            Some(pos) if pos > 0 => trimmed[..pos].to_string(),
            _ => "/".to_string(),
        }
    }
}
