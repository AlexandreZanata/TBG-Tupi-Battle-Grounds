#!/usr/bin/env python3
"""Local loopback preview of a site with the production security headers.

Serves the site directory plus its manifest mappings straight from the
repository, so edits show on reload. Headers mirror web/deploy/Caddyfile.web.template
so a CSP violation appears in the browser console before deployment.
With --public-status the admin preview may read the backend's public
/health and /api/version (GET only, small, read-only); nothing else is proxied.
"""

import argparse
import json
import mimetypes
import sys
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import unquote, urlsplit

from sites import SITES, files

CSP = {
    "landing": "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; "
    "connect-src 'self'; manifest-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
    "admin-public": "default-src 'none'; style-src 'self'; img-src 'self'; font-src 'self'; "
    "base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
    "admin-preview": "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; "
    "connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
}
PUBLIC_STATUS = ("/health", "/api/version")
STATUS_LIMIT = 4096
USER_AGENT = "tbg-web-preview/1 (local admin preview; public status only)"
mimetypes.add_type("text/javascript", ".js")
mimetypes.add_type("image/webp", ".webp")
mimetypes.add_type("font/woff2", ".woff2")


def handler(site, mapping, public_status):
    class Handler(BaseHTTPRequestHandler):
        server_version = "tbg-preview"

        def _headers(self, status, ctype, length):
            self.send_response(status)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(length))
            self.send_header("Cache-Control", "no-store")
            self.send_header("Content-Security-Policy", CSP[site.name])
            self.send_header("X-Content-Type-Options", "nosniff")
            self.send_header("Referrer-Policy", "strict-origin-when-cross-origin")
            self.send_header("X-Frame-Options", "DENY")
            if site.name != "landing":
                self.send_header("X-Robots-Tag", "noindex, nofollow")
            self.end_headers()

        def _send(self, status, body, ctype):
            self._headers(status, ctype, len(body))
            if self.command != "HEAD":
                self.wfile.write(body)

        def _proxy_status(self, path):
            # Cloudflare refuses urllib's default agent, so name this tool honestly.
            request = urllib.request.Request(public_status + path, headers={"User-Agent": USER_AGENT})
            try:
                with urllib.request.urlopen(request, timeout=5) as reply:
                    body = reply.read(STATUS_LIMIT + 1)[:STATUS_LIMIT]
                    self._send(reply.status, body, reply.headers.get_content_type())
            except urllib.error.HTTPError as error:
                self._send(error.code, b'{"error":"public status refused"}', "application/json")
            except (urllib.error.URLError, TimeoutError, OSError):
                self._send(502, b'{"error":"public status unreachable"}', "application/json")

        def do_GET(self):
            path = unquote(urlsplit(self.path).path)
            if public_status and path in PUBLIC_STATUS:
                return self._proxy_status(path)
            if site.name == "admin-preview" and path == "/preview.json":
                # Lets the preview label its environment and data sources honestly.
                body = json.dumps({"mode": "local-preview", "publicStatus": bool(public_status)}).encode()
                return self._send(200, body, "application/json")
            rel = path.lstrip("/")
            if rel == "" or rel.endswith("/"):
                rel += "index.html"
            source = mapping.get(rel)
            if source is None:
                missing = mapping.get("404.html")
                body = missing.read_bytes() if missing else b"Not found"
                return self._send(404, body, "text/html; charset=utf-8" if missing else "text/plain")
            ctype = mimetypes.guess_type(rel)[0] or "application/octet-stream"
            if ctype.startswith("text/") or ctype in ("application/json", "image/svg+xml"):
                ctype += "; charset=utf-8"
            self._send(200, source.read_bytes(), ctype)

        do_HEAD = do_GET

        def log_message(self, fmt, *args):
            sys.stderr.write("%s %s\n" % (self.command, fmt % args))

    return Handler


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("site", choices=sorted(SITES))
    parser.add_argument("--port", type=int, default=0, help="0 picks a free port")
    parser.add_argument("--public-status", metavar="URL", help="admin preview only, e.g. https://api.tupibg.com")
    args = parser.parse_args()
    site = SITES[args.site]
    if args.public_status and site.name != "admin-preview":
        parser.error("--public-status is only for the admin preview")
    # The mapping is read once; restart after adding or renaming files.
    server = ThreadingHTTPServer(("127.0.0.1", args.port), handler(site, files(site), args.public_status))
    print(f"{site.name} preview: http://127.0.0.1:{server.server_port}/  (Ctrl+C to stop)", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
