from pydantic import (
    BaseModel,
    BaseSettings,
    PyObject,
    RedisDsn,
    Field
)

import os
from pathlib import Path

def base_path():
    self_path = Path(os.path.dirname(os.path.abspath(__file__)))
    base_path = self_path.parent.parent
    return base_path

def default_data_dir():
    return os.path.join(base_path(), 'data/oas')

def default_frontend_path():
    return os.path.join(base_path(), 'frontend/dist')

class Settings(BaseSettings):
    # general settings
    storage_path: str = default_data_dir()
    model: str = 'vosk-model-de-0.21'
    model_path: str = ''
    log_level: str = 'info'

    oas_url: str = 'http://admin:password@localhost:8080/api/v1'
    redis_url: RedisDsn = 'redis://localhost:6379/0'

    # set to 1 to enable development mode
    # (hot reload code on changes)
    oas_dev: bool = False

    # config parsing
    class Config:
        env_file = '.env'

    # helper functions
    def url(self, path):
        return self.oas_url + path


config = Settings()
