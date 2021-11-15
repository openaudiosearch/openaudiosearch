import logging
import time
import os
from urllib.parse import urlparse, urlunparse
from pathlib import Path

DEFAULT_OAS_URL = 'http://admin:password@localhost:8080/api/v1'

def base_path():
    self_path = Path(os.path.dirname(os.path.abspath(__file__)))
    base_path = self_path.parent.parent
    return base_path

def default_storage_dir():
    return os.path.join(base_path(), 'data/oas')

class Config(object):
    def __init__(self):
        self.base_url = os.environ.get("OAS_URL") or DEFAULT_OAS_URL
        self.storage_path = os.environ.get("OAS_STORAGE") or default_storage_dir() 
        self.log_level = os.environ.get('LOG', 'INFO')
        self.log_file = os.environ.get('OAS_LOGFILE') or os.path.join(self.storage_path, 'oas-worker.log')
        self.model = 'vosk-model-de-0.6'
        self.model_path = os.path.join(self.storage_path, "models")

        try:
            self.base_url_parsed = urlparse(self.base_url, 'http')
        except BaseException as err:
            logging.error(f"Failed to parse OAS_URL: {err}")

    def local_dir(self, path):
        return os.path.join(self.storage_path, path)


config = Config()
