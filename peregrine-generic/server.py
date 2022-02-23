# -*- coding: utf-8 -*-
import sys
import http.server
from http.server import HTTPServer, BaseHTTPRequestHandler
import socketserver

port = int(sys.argv[1])
if port == 0:
	sys.exit(0)

socketserver.TCPServer.allow_reuse_address = True
Handler = http.server.SimpleHTTPRequestHandler

Handler.extensions_map={
	'.html': 'text/html',
        '.png': 'image/png',
	'.jpg': 'image/jpg',
	'.css':	'text/css',
	'.js':	'application/x-javascript',
        '.wasm': 'application/wasm',
	'': 'application/octet-stream', # Default
    }

httpd = socketserver.TCPServer(("", port), Handler)

print("serving at port", port)
httpd.serve_forever()
