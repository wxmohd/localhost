use crate::error::{Result, ServerError};
use crate::http::{Request, Response, StatusCode};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// CGI script executor
pub struct CgiExecutor;

impl CgiExecutor {
    /// Executes a CGI script and returns the response
    pub fn execute(request: &Request, script_path: &str, interpreter: &str) -> Result<Response> {
        let path = Path::new(script_path);

        // Check if script exists
        if !path.exists() {
            return Err(ServerError::NotFound);
        }

        // Build environment variables
        let env_vars = Self::build_env(request, script_path);

        // Get the script's directory for working directory
        let working_dir = path.parent().unwrap_or(Path::new("."));

        // Execute the CGI script
        let mut child = Command::new(interpreter)
            .arg(script_path)
            .envs(env_vars)
            .current_dir(working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ServerError::Cgi(format!("Failed to spawn CGI process: {}", e)))?;

        // Write request body to stdin
        if !request.body.is_empty() {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(&request.body)
                    .map_err(|e| ServerError::Cgi(format!("Failed to write to CGI stdin: {}", e)))?;
            }
        }

        // Wait for the process to complete
        let output = child.wait_with_output()
            .map_err(|e| ServerError::Cgi(format!("CGI process failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("CGI error: {}", stderr);
            return Err(ServerError::Cgi(format!("CGI script failed: {}", stderr)));
        }

        // Parse CGI output
        Self::parse_cgi_output(&output.stdout)
    }

    /// Builds CGI environment variables
    fn build_env(request: &Request, script_path: &str) -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Standard CGI variables
        env.insert("REQUEST_METHOD".to_string(), request.method.to_string());
        env.insert("SCRIPT_FILENAME".to_string(), script_path.to_string());
        env.insert("SCRIPT_NAME".to_string(), request.path.clone());
        env.insert("PATH_INFO".to_string(), request.path.clone());
        env.insert("QUERY_STRING".to_string(), Self::build_query_string(&request.query));
        env.insert("SERVER_PROTOCOL".to_string(), request.version.clone());
        env.insert("SERVER_SOFTWARE".to_string(), "localhost/0.1.0".to_string());
        env.insert("GATEWAY_INTERFACE".to_string(), "CGI/1.1".to_string());

        // Content headers
        if let Some(content_type) = request.content_type() {
            env.insert("CONTENT_TYPE".to_string(), content_type.to_string());
        }
        if let Some(content_length) = request.content_length() {
            env.insert("CONTENT_LENGTH".to_string(), content_length.to_string());
        }

        // HTTP headers as environment variables
        for (name, values) in request.headers.iter() {
            let env_name = format!("HTTP_{}", name.to_uppercase().replace('-', "_"));
            if let Some(value) = values.first() {
                env.insert(env_name, value.clone());
            }
        }

        // Host information
        if let Some(host) = request.host() {
            env.insert("HTTP_HOST".to_string(), host.to_string());
            env.insert("SERVER_NAME".to_string(), host.split(':').next().unwrap_or(host).to_string());
        }

        env
    }

    /// Builds query string from parameters
    fn build_query_string(params: &HashMap<String, String>) -> String {
        params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Parses CGI output into an HTTP response
    fn parse_cgi_output(output: &[u8]) -> Result<Response> {
        let output_str = String::from_utf8_lossy(output);

        // Find the header/body separator
        let (headers_part, body) = if let Some(pos) = output_str.find("\r\n\r\n") {
            (&output_str[..pos], &output[pos + 4..])
        } else if let Some(pos) = output_str.find("\n\n") {
            (&output_str[..pos], &output[pos + 2..])
        } else {
            // No headers, treat entire output as body
            return Ok(Response::ok()
                .content_type("text/html")
                .body(output.to_vec()));
        };

        // Parse headers
        let mut response = Response::new(StatusCode::Ok);
        let mut has_content_type = false;

        for line in headers_part.lines() {
            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim();

                match name.to_lowercase().as_str() {
                    "status" => {
                        // Parse status code from "Status: 200 OK" format
                        if let Some(code) = value.split_whitespace().next() {
                            if let Ok(code_num) = code.parse::<u16>() {
                                if let Some(status) = StatusCode::from_code(code_num) {
                                    response.status = status;
                                }
                            }
                        }
                    }
                    "content-type" => {
                        response.headers.set("Content-Type", value);
                        has_content_type = true;
                    }
                    "location" => {
                        response.headers.set("Location", value);
                        if response.status == StatusCode::Ok {
                            response.status = StatusCode::Found;
                        }
                    }
                    _ => {
                        response.headers.set(name, value);
                    }
                }
            }
        }

        // Default content type
        if !has_content_type {
            response.headers.set("Content-Type", "text/html");
        }

        // Set body
        response.body = body.to_vec();
        response.headers.set("Content-Length", &response.body.len().to_string());

        Ok(response)
    }
}
