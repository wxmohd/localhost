# localhost

A robust HTTP/1.1 web server implementation in Rust, built from scratch without using high-level networking frameworks.

## Overview

This project implements a fully functional HTTP server using only the Rust standard library and `libc` for low-level system calls. It features a single-threaded, non-blocking event loop architecture that handles multiple concurrent connections efficiently.

**Key highlights:**
- No tokio, async-std, or other async runtimes
- Single process, single thread design
- Poll-based I/O multiplexing (single poll call per event loop iteration)
- NGINX-style configuration file

## Features

- **HTTP/1.1 Protocol** — Full compliance with HTTP/1.1 specification
- **HTTP Methods** — GET, POST, DELETE support
- **Non-blocking I/O** — All sockets are non-blocking with poll-based multiplexing
- **Static File Serving** — Serve HTML, CSS, JS, images with correct MIME types
- **Directory Listing** — Configurable autoindex for directories
- **CGI Support** — Execute Python scripts (extensible to other languages)
- **File Uploads** — Handle multipart/form-data uploads
- **Sessions & Cookies** — Automatic session management with HttpOnly cookies
- **Virtual Hosting** — Multiple server blocks with different server_name
- **HTTP Redirects** — 301/302 redirections
- **Custom Error Pages** — Styled pages for 400, 403, 404, 405, 413, 500
- **Request Timeouts** — Configurable connection timeouts
- **Body Size Limits** — Configurable client_max_body_size
- **Chunked Transfer** — Support for chunked request encoding

## Project Structure

```
localhost/
├── Cargo.toml                 # Project manifest (libc dependency)
├── config/
│   ├── default.conf           # Main server configuration
│   └── duplicate_port.conf    # Test config for error detection
├── src/
│   ├── main.rs                # Entry point, config loading
│   ├── config/                # Configuration parsing
│   │   ├── mod.rs
│   │   ├── parser.rs          # NGINX-style config parser
│   │   ├── route.rs           # Route/location configuration
│   │   └── server_config.rs   # Server block configuration
│   ├── server/                # Core server components
│   │   ├── mod.rs
│   │   ├── connection.rs      # Connection state machine
│   │   ├── epoll.rs           # Poll-based I/O multiplexing
│   │   ├── event_loop.rs      # Main event loop
│   │   └── listener.rs        # TCP listener wrapper
│   ├── http/                  # HTTP protocol implementation
│   │   ├── mod.rs
│   │   ├── headers.rs         # Header parsing/building
│   │   ├── method.rs          # HTTP methods enum
│   │   ├── request.rs         # Request parsing
│   │   ├── response.rs        # Response building
│   │   └── status.rs          # Status codes
│   ├── router/                # Request routing
│   │   ├── mod.rs
│   │   ├── directory.rs       # Directory listing generator
│   │   ├── handler.rs         # Main request handler
│   │   ├── redirect.rs        # HTTP redirects
│   │   └── static_files.rs    # Static file serving
│   ├── cgi/                   # CGI execution
│   │   ├── mod.rs
│   │   └── executor.rs        # CGI process management
│   ├── session/               # Session management
│   │   ├── mod.rs
│   │   ├── cookie.rs          # Cookie parsing
│   │   └── store.rs           # In-memory session store
│   └── error/                 # Error handling
│       ├── mod.rs
│       └── pages.rs           # Error page generation
├── www/                       # Web root directory
│   ├── index.html             # Default homepage
│   ├── cgi-bin/
│   │   └── hello.py           # Sample Python CGI script
│   ├── errors/                # Custom error pages
│   │   ├── 400.html
│   │   ├── 403.html
│   │   ├── 404.html
│   │   ├── 405.html
│   │   ├── 413.html
│   │   └── 500.html
│   ├── files/                 # Directory listing demo
│   │   └── test.txt
│   └── uploads/               # File upload directory
├── audit_tests.ps1            # Automated audit test script
└── AUDIT_COMPLIANCE.md        # Detailed audit compliance document
```

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

## Running

```bash
# With default config (config/default.conf)
./target/release/localhost

# With custom config
./target/release/localhost path/to/config.conf

# Or using cargo
cargo run --release
cargo run --release -- path/to/config.conf
```

The server will start and display:
```
localhost HTTP Server v0.1.0
Loading configuration from: config/default.conf

Server configuration:
  - Host: 127.0.0.1
    Ports: [8080]
    Root: ./www
    Routes: 6

Starting server...
Listening on 127.0.0.1:8080
Server started, waiting for connections...
```

## Configuration

The server uses an NGINX-style configuration format:

```nginx
server {
    listen 8080
    server_name localhost
    host 127.0.0.1
    root ./www
    
    client_max_body_size 10M
    timeout 60
    
    error_page 400 /errors/400.html
    error_page 403 /errors/403.html
    error_page 404 /errors/404.html
    error_page 405 /errors/405.html
    error_page 413 /errors/413.html
    error_page 500 /errors/500.html
    
    location / {
        methods GET POST DELETE
        index index.html
        autoindex off
    }
    
    location /uploads {
        methods GET POST DELETE
        upload_dir ./www/uploads
    }
    
    location /cgi-bin {
        methods GET POST
        cgi .py python
    }
    
    location /files {
        methods GET
        autoindex on
    }
    
    location /redirect-test {
        return https://example.com
    }
}

# Virtual host example
server {
    listen 8080
    server_name test.local
    host 127.0.0.1
    root ./www
    
    location / {
        methods GET POST
        index index.html
    }
}
```

### Configuration Directives

| Directive | Description |
|-----------|-------------|
| `listen` | Port to listen on |
| `server_name` | Virtual host name (matched against Host header) |
| `host` | IP address to bind to |
| `root` | Document root directory |
| `client_max_body_size` | Maximum request body size (e.g., 10M, 1G) |
| `timeout` | Connection timeout in seconds |
| `error_page` | Custom error page path |
| `location` | Route configuration block |
| `methods` | Allowed HTTP methods for route |
| `index` | Default index file |
| `autoindex` | Enable directory listing (on/off) |
| `cgi` | CGI handler (extension interpreter) |
| `upload_dir` | Directory for file uploads |
| `return` | HTTP redirect URL |

## Testing

### Quick Test
```bash
# Test basic functionality
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/files/
curl http://127.0.0.1:8080/cgi-bin/hello.py?name=World
curl -X POST -F "file=@test.txt" http://127.0.0.1:8080/uploads/
```

### Automated Audit Tests
```powershell
# Run the full audit test suite
powershell -ExecutionPolicy Bypass -File .\audit_tests.ps1
```

### Stress Testing
```bash
# Using siege (Linux/WSL)
siege -b http://127.0.0.1:8080/

# Expected: 99.5%+ availability
```

### Test Duplicate Port Detection
```bash
./target/release/localhost config/duplicate_port.conf
# Should output: "Configuration error: Duplicate port binding..."
```

## API Examples

### GET Request
```bash
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/files/
```

### POST File Upload
```bash
curl -X POST -F "file=@document.pdf" http://127.0.0.1:8080/uploads/
# Response: {"status":"ok","files":["document.pdf"]}
```

### DELETE File
```bash
curl -X DELETE http://127.0.0.1:8080/uploads/document.pdf
# Response: {"status":"ok","message":"File deleted"}
```

### CGI with Parameters
```bash
curl "http://127.0.0.1:8080/cgi-bin/hello.py?name=Alice"
```

### Virtual Hosting
```bash
curl -H "Host: test.local" http://127.0.0.1:8080/
```

## Requirements

- **Rust** 2021 edition (1.56+)
- **Python 3** (for CGI scripts)
- **Windows** or **Linux/macOS**

## Architecture

### Event Loop
The server uses a single-threaded event loop with poll-based I/O multiplexing:

1. **Poll** — Single `Poller::wait()` call checks all registered sockets
2. **Accept** — New connections are accepted and registered
3. **Read** — Data is read from ready sockets (non-blocking)
4. **Process** — HTTP requests are parsed and routed
5. **Write** — Responses are written to ready sockets (non-blocking)
6. **Cleanup** — Timed-out or errored connections are removed

### Connection State Machine
```
New → Reading → Processing → Writing → Done
                    ↓
                  Error
```

## License

MIT
