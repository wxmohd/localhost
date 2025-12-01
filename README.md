# localhost

A HTTP/1.1 web server implementation in Rust.

## Overview

This project implements a fully functional HTTP server from scratch, handling static file serving, CGI execution, sessions, and more — all using a single-threaded, non-blocking event loop.

## Features

- **HTTP/1.1 compliant** — GET, POST, DELETE methods
- **Non-blocking I/O** — epoll-based event loop
- **Single process, single thread** — efficient resource usage
- **Static file serving** — serve HTML, CSS, JS, images
- **CGI support** — execute Python scripts
- **Sessions & cookies** — basic session management
- **File uploads** — handle multipart form data
- **Chunked transfer encoding** — support for chunked requests
- **Custom error pages** — 400, 403, 404, 405, 413, 500
- **Configurable** — flexible configuration file

## Project Structure

```
localhost/
├── src/
│   ├── main.rs           # Entry point
│   ├── config/           # Configuration parsing
│   ├── server/           # Server, epoll, connections
│   ├── http/             # HTTP protocol (request, response, parsing)
│   ├── router/           # Request routing, static files, redirects
│   ├── cgi/              # CGI execution
│   ├── session/          # Cookies and session management
│   └── error/            # Error handling and pages
├── config/
│   └── default.conf      # Default server configuration
├── www/
│   ├── index.html        # Default index page
│   ├── errors/           # Custom error pages
│   └── cgi-bin/          # CGI scripts
└── tests/
    └── integration_tests.rs
```

## Building

```bash
cargo build --release
```

## Running

```bash
# With default config
cargo run

# With custom config
cargo run -- path/to/config.conf
```

## Configuration

Example configuration:

```
server {
    listen 8080
    server_name localhost
    root ./www
    
    error_page 404 /errors/404.html
    client_max_body_size 10M
    
    location / {
        methods GET POST
        index index.html
        autoindex off
    }
    
    location /cgi-bin {
        methods GET POST
        cgi .py /usr/bin/python3
    }
}
```

## Testing

```bash
# Run tests
cargo test

# Stress test with siege
siege -b http://localhost:8080
```

## Requirements

- Rust 1.70+
- Linux (for epoll) or Windows (for IOCP equivalent)

## License

MIT
