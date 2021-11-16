import requests
from urllib.parse import urlparse
from hashlib import sha256
from base64 import b32encode

def pretty_bytes(num, suffix='B'):
    """
    Format bytes as human-readable string
    """
    for unit in ['', 'Ki', 'Mi', 'Gi', 'Ti', 'Pi', 'Ei', 'Zi']:
        if abs(num) < 1024.0:
            return "%3.1f%s%s" % (num, unit, suffix)
        num /= 1024.0
    return "%.1f%s%s" % (num, 'Yi', suffix)


def download_file(url, local_filename, on_progress=None, on_headers=None):
    with requests.get(url, stream=True) as res:
        res.raise_for_status()
        if on_headers:
            on_headers(res.headers)

        chunk_size = 1024*64
        total_size = int(res.headers.get('content-length', 0))
        download_size = 0
        with open(local_filename, 'wb') as f:
            for chunk in res.iter_content(chunk_size=chunk_size):
                download_size += len(chunk)
                # If you have chunk encoded response uncomment if
                # and set chunk_size parameter to None.
                # if chunk:
                f.write(chunk)
                if on_progress and download_size and total_size:
                    on_progress(download_size / total_size)
                f.flush()


def ensure_dir(path: str, parent=True):
    path = Path(path)
    if parent:
        path = path.parent
    os.makedirs(path, exist_ok=True)


def url_to_path(url: str) -> str:
    parsed_url = urlparse(url)

    url_hash = sha256(url.encode("utf-8")).digest()
    url_hash = b32encode(url_hash)
    url_hash = url_hash.lower().decode("utf-8")

    target_name = f"{parsed_url.netloc}/{url_hash[:2]}/{url_hash[2:]}"
    return target_name


def find_in_dict(input, keys):
    keys = keys.split(".")
    cursor = input
    for key in keys:
        cursor = cursor.get(key)
        if cursor is None:
            return None
    return cursor

