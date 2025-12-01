use super::headers::Headers;
use super::method::Method;
use super::request::Request;
use crate::error::{Result, ServerError};

/// HTTP request parser
pub struct RequestParser;

impl RequestParser {
    /// Parses a complete HTTP request from bytes
    pub fn parse(data: &[u8]) -> Result<Request> {
        let data_str = std::str::from_utf8(data)
            .map_err(|_| ServerError::Parse("Invalid UTF-8 in request".to_string()))?;

        // Find the end of headers
        let header_end = data_str
            .find("\r\n\r\n")
            .ok_or_else(|| ServerError::Parse("Incomplete headers".to_string()))?;

        let header_section = &data_str[..header_end];
        let body_start = header_end + 4;

        // Parse request line and headers
        let mut lines = header_section.lines();
        
        // Parse request line: METHOD PATH HTTP/VERSION
        let request_line = lines
            .next()
            .ok_or_else(|| ServerError::Parse("Missing request line".to_string()))?;

        let mut parts = request_line.split_whitespace();
        
        let method_str = parts
            .next()
            .ok_or_else(|| ServerError::Parse("Missing method".to_string()))?;
        
        let method: Method = method_str
            .parse()
            .map_err(|_| ServerError::Parse(format!("Invalid method: {}", method_str)))?;

        let path = parts
            .next()
            .ok_or_else(|| ServerError::Parse("Missing path".to_string()))?;

        let version = parts
            .next()
            .ok_or_else(|| ServerError::Parse("Missing HTTP version".to_string()))?
            .to_string();

        // Parse headers
        let mut headers = Headers::new();
        for line in lines {
            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim();
                headers.add(name, value);
            }
        }

        // Create request
        let mut request = Request::new(method, path);
        request.version = version;
        request.headers = headers;

        // Extract body
        if body_start < data.len() {
            request.body = data[body_start..].to_vec();
        }

        Ok(request)
    }

    /// Checks if we have received complete headers
    pub fn has_complete_headers(data: &[u8]) -> bool {
        if let Ok(s) = std::str::from_utf8(data) {
            s.contains("\r\n\r\n")
        } else {
            false
        }
    }

    /// Gets the expected body length from headers
    pub fn get_content_length(data: &[u8]) -> Option<usize> {
        if let Ok(s) = std::str::from_utf8(data) {
            for line in s.lines() {
                let lower = line.to_lowercase();
                if lower.starts_with("content-length:") {
                    return line[15..].trim().parse().ok();
                }
            }
        }
        None
    }

    /// Checks if transfer encoding is chunked
    pub fn is_chunked(data: &[u8]) -> bool {
        if let Ok(s) = std::str::from_utf8(data) {
            for line in s.lines() {
                let lower = line.to_lowercase();
                if lower.starts_with("transfer-encoding:") && lower.contains("chunked") {
                    return true;
                }
            }
        }
        false
    }

    /// Checks if we have a complete request (headers + body)
    pub fn is_complete(data: &[u8]) -> bool {
        if !Self::has_complete_headers(data) {
            return false;
        }

        // Find header end
        if let Ok(s) = std::str::from_utf8(data) {
            if let Some(header_end) = s.find("\r\n\r\n") {
                let body_start = header_end + 4;
                
                // Check content length
                if let Some(content_length) = Self::get_content_length(data) {
                    return data.len() >= body_start + content_length;
                }

                // Check chunked encoding
                if Self::is_chunked(data) {
                    // Look for final chunk marker: 0\r\n\r\n
                    return s[body_start..].contains("0\r\n\r\n");
                }

                // No body expected
                return true;
            }
        }

        false
    }

    /// Decodes chunked transfer encoding
    pub fn decode_chunked(data: &[u8]) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        let mut pos = 0;

        while pos < data.len() {
            // Find chunk size line
            let line_end = Self::find_crlf(&data[pos..])
                .ok_or_else(|| ServerError::Parse("Invalid chunked encoding".to_string()))?;
            
            let size_str = std::str::from_utf8(&data[pos..pos + line_end])
                .map_err(|_| ServerError::Parse("Invalid chunk size".to_string()))?;
            
            let chunk_size = usize::from_str_radix(size_str.trim(), 16)
                .map_err(|_| ServerError::Parse("Invalid chunk size".to_string()))?;

            pos += line_end + 2; // Skip size line and CRLF

            if chunk_size == 0 {
                break; // Final chunk
            }

            // Read chunk data
            if pos + chunk_size > data.len() {
                return Err(ServerError::Parse("Incomplete chunk".to_string()));
            }

            result.extend_from_slice(&data[pos..pos + chunk_size]);
            pos += chunk_size + 2; // Skip chunk data and trailing CRLF
        }

        Ok(result)
    }

    /// Finds CRLF in data
    fn find_crlf(data: &[u8]) -> Option<usize> {
        for i in 0..data.len().saturating_sub(1) {
            if data[i] == b'\r' && data[i + 1] == b'\n' {
                return Some(i);
            }
        }
        None
    }
}
