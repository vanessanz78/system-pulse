from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parent


class FreshStaticHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ROOT), **kwargs)

    def end_headers(self):
        self.send_header("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0")
        self.send_header("Pragma", "no-cache")
        self.send_header("Expires", "0")
        super().end_headers()


if __name__ == "__main__":
    port = int(os.environ.get("PORT", "3000"))
    server = ThreadingHTTPServer(("0.0.0.0", port), FreshStaticHandler)
    print(f"System Pulse website running on port {port}")
    server.serve_forever()
