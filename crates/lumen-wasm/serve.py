#!/usr/bin/env python3
"""HTTP server to serve LÚMEN WASM playground with proper headers."""
import http.server
import socketserver
import os
import sys
import mimetypes

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
DIR = os.path.dirname(os.path.abspath(__file__))

# Ensure correct MIME types for WASM
mimetypes.add_type('application/wasm', '.wasm')
mimetypes.add_type('application/javascript', '.js')
mimetypes.add_type('text/html', '.html')

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIR, **kwargs)

    def end_headers(self):
        # COOP/COEP headers for SharedArrayBuffer support (needed by some WASM runtimes)
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        # Cache control for WASM modules
        if self.path.endswith('.wasm'):
            self.send_header('Cache-Control', 'public, max-age=0, must-revalidate')
        super().end_headers()

    def log_message(self, format, *args):
        print(f"[{self.log_date_time_string()}] {args[0]} {args[1]} {args[2]}")

with socketserver.TCPServer(("", PORT), Handler) as httpd:
    print(f"╔══════════════════════════════════════════╗")
    print(f"║   LÚMEN Playground — WASM Runtime        ║")
    print(f"║                                          ║")
    print(f"║   ▶ http://localhost:{PORT}/web/index.html  ║")
    print(f"║                                          ║")
    print(f"║   Ctrl+C para detener                    ║")
    print(f"╚══════════════════════════════════════════╝")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nServidor detenido.")
        httpd.server_close()
