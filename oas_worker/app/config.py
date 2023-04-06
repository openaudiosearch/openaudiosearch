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
        self.cache_path = os.environ.get("OAS_CACHE_DIR") or os.path.join(self.storage_path, "cache")
        self.log_level = os.environ.get('LOG', 'INFO')
        self.log_file = os.environ.get('OAS_LOGFILE') or os.path.join(self.storage_path, 'oas-worker.log')
        self.model = 'vosk-model-de-0.21'
        self.recase_model = 'vosk-recasepunc-de-0.21/checkpoint'
        self.model_path = os.path.join(self.storage_path, "models")

        try:
            self.base_url_parsed = urlparse(self.base_url, 'http')
        except BaseException as err:
            logging.error(f"Failed to parse OAS_URL: {err}")


    def local_dir(self, path):
        return os.path.join(self.storage_path, path)

    def cache_dir(self, path):
        return os.path.join(self.cache_path, path)


config = Config()
