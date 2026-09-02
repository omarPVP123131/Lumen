#!/usr/bin/env python3
"""Servidor HTTP para servir el Playground Web y Landing de LÚMEN con headers correctos."""
import http.server
import socketserver
import os
import sys
import mimetypes

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
DIR = os.path.dirname(os.path.abspath(__file__))

# Asegurar tipos MIME correctos
mimetypes.add_type('application/wasm', '.wasm')
mimetypes.add_type('application/javascript', '.js')
mimetypes.add_type('text/html', '.html')
mimetypes.add_type('text/css', '.css')
mimetypes.add_type('application/json', '.json')

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIR, **kwargs)

    def do_GET(self):
        # Rutas amigables
        path = self.path.split('?')[0]
        if path in ('/', '/playground', '/ide'):
            self.send_response(302)
            self.send_header('Location', '/web/playground.html')
            self.end_headers()
            return
        elif path in ('/portal', '/home', '/docs'):
            self.send_response(302)
            self.send_header('Location', '/web/index.html')
            self.end_headers()
            return
        super().do_GET()

    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        if self.path.endswith('.wasm'):
            self.send_header('Cache-Control', 'public, max-age=0, must-revalidate')
        super().end_headers()

    def log_message(self, format, *args):
        print(f"[{self.log_date_time_string()}] {args[0]} {args[1]} {args[2]}")

if __name__ == '__main__':
    with socketserver.TCPServer(("", PORT), Handler) as httpd:
        print(f"╔══════════════════════════════════════════════════════════════════════╗")
        print(f"║   🚀 LÚMEN Web Server & Playground — WASM Runtime v2.4.6            ║")
        print(f"║                                                                      ║")
        print(f"║   ⚡ Playground Pro (Full IDE):                                      ║")
        print(f"║      ▶ http://localhost:{PORT}/web/playground.html                      ║")
        print(f"║                                                                      ║")
        print(f"║   🏠 Portal Principal & Documentación:                               ║")
        print(f"║      ▶ http://localhost:{PORT}/web/index.html                           ║")
        print(f"║                                                                      ║")
        print(f"║   Presiona Ctrl+C para detener el servidor                           ║")
        print(f"╚══════════════════════════════════════════════════════════════════════╝")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nServidor detenido.")
            httpd.server_close()
