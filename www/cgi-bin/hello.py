#!/usr/bin/env python3
import os
import sys
from datetime import datetime

# CGI headers
print("Content-Type: text/html")
print()

# Get environment variables
method = os.environ.get('REQUEST_METHOD', 'GET')
query = os.environ.get('QUERY_STRING', '')
path = os.environ.get('PATH_INFO', '/')

# Parse query string
params = {}
if query:
    for pair in query.split('&'):
        if '=' in pair:
            key, value = pair.split('=', 1)
            params[key] = value

name = params.get('name', 'World')

# Generate HTML
print(f"""<!DOCTYPE html>
<html>
<head>
    <title>CGI Hello</title>
    <style>
        body {{
            font-family: sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        .card {{
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        h1 {{ color: #333; }}
        .info {{ color: #666; margin: 10px 0; }}
        form {{ margin-top: 20px; }}
        input {{ padding: 10px; margin-right: 10px; border: 1px solid #ddd; border-radius: 5px; }}
        button {{ padding: 10px 20px; background: #4facfe; color: white; border: none; border-radius: 5px; cursor: pointer; }}
    </style>
</head>
<body>
    <div class="card">
        <h1>Hello, {name}!</h1>
        <p class="info">Method: {method}</p>
        <p class="info">Time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>
        <p class="info">Path: {path}</p>
        
        <form method="get">
            <input type="text" name="name" placeholder="Enter your name">
            <button type="submit">Say Hello</button>
        </form>
    </div>
</body>
</html>
""")
