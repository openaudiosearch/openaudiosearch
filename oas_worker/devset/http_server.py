import http.server
from threading import Thread, current_thread
from functools import partial
from sys import stderr
from os.path import abspath

def serve_directory(directory=".",port=6650):
    hostname = "localhost"
    directory = abspath(directory)
    handler = partial(http.server.SimpleHTTPRequestHandler, directory=directory)
    httpd = http.server.HTTPServer((hostname, port), handler, False)
    httpd.server_bind()

    address = "http://%s:%d" % (httpd.server_name, httpd.server_port)

    httpd.server_activate()

    def serve_forever(httpd):
        with httpd:  # to make sure httpd.server_close is called
            print(f"server listening {address} and serving {directory}")
            httpd.serve_forever()
            print("server closed")

    thread = Thread(target=serve_forever, args=(httpd, ))
    thread.setDaemon(True)
    thread.start()

    return httpd, address


def directory_handler(directory):
    class Handler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=directory, **kwargs)

    return Handler
