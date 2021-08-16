# -*- coding: utf-8 -*-
import http.server
from http.server import HTTPServer, BaseHTTPRequestHandler
import socketserver

PORT = 8000

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

httpd = socketserver.TCPServer(("", PORT), Handler)

print("serving at port", PORT)
httpd.serve_forever()

