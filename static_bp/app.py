from http import server
import socketserver

class MyHTTPRequestHandler(server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Connection', 'close')
        self.send_header('Expires', '-1')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header('Cross-Origin-Resource-Policy', 'cross-origin')

        server.SimpleHTTPRequestHandler.end_headers(self)

PORT = 8000
Handler = MyHTTPRequestHandler

if __name__ == '__main__':
    with socketserver.TCPServer(("", PORT), Handler) as httpd:
        print("serving at port", PORT)
        httpd.serve_forever()