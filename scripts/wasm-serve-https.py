#!/usr/bin/env python3
# serve-https — stdlib TLS file server for the wasm serve-https lane
# (owner ruling 2026-10-16). Serves a directory (typically the assembled
# web-demo dir) over HTTPS on 0.0.0.0 so LAN clients get a secure context
# and WebGPU without tunnels. Independent of the gate server: static files
# only, no beacon logic here. Certs from scripts/wasm-serve-https-gen.sh.
import http.server, socketserver, ssl, sys, os
if len(sys.argv) != 5:
    sys.exit("usage: serve-https.py DIR PORT CERT KEY")
root, port, cert, key = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
if not os.path.isdir(root):
    sys.exit("SERVE-HTTPS FAIL: missing dir %s -- build visiaengine-wasm-serve once" % root)
for f in (cert, key):
    if not os.path.isfile(f):
        sys.exit("SERVE-HTTPS FAIL: missing %s -- run scripts/wasm-serve-https-gen.sh" % f)
class H(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map,
                      'wasm': 'application/wasm'}
    def __init__(self, *a, **k):
        super().__init__(*a, directory=root, **k)
class S(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True
try:
    srv = S(('0.0.0.0', port), H)
except OSError as e:
    sys.exit("SERVE-HTTPS BUSY %s" % e)
ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ctx.load_cert_chain(cert, key)
srv.socket = ctx.wrap_socket(srv.socket, server_side=True)
print("SERVE-HTTPS LISTENING %d" % port, flush=True)
srv.serve_forever()
