import requests
import os
from base64 import b32encode
from uuid import uuid4

def uuid():
    """
    Generate a base32 encoded UUID string"
    """
    B32_LEN = 26
    encoded = b32encode(uuid4().bytes)[:B32_LEN]
    return encoded.lower().decode('utf-8') 

def pretty_bytes(num, suffix='B'):
    """
    Format bytes as human-readable string
    """
    for unit in ['','Ki','Mi','Gi','Ti','Pi','Ei','Zi']:
        if abs(num) < 1024.0:
            return "%3.1f%s%s" % (num, unit, suffix)
        num /= 1024.0
    return "%.1f%s%s" % (num, 'Yi', suffix)


def download_file(url, local_filename,on_progress=None, on_headers=None):
    with requests.get(url, stream=True) as res:
        res.raise_for_status()
        if on_headers:
            on_headers(res.headers)

        chunk_size=1024*64
        total_size = int(res.headers.get('content-length', 0))
        download_size = 0
        with open(local_filename, 'wb') as f:
            for chunk in res.iter_content(chunk_size=chunk_size): 
                download_size += len(chunk)
                # If you have chunk encoded response uncomment if
                # and set chunk_size parameter to None.
                #if chunk: 
                f.write(chunk)
                if on_progress and download_size and total_size:
                    on_progress(download_size / total_size)
                f.flush()