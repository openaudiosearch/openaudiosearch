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
    frontend_path: str = default_frontend_path()
    model: str = 'vosk-model-de-0.6'
    model_path: str = ''
    log_level: str = 'info'

    # server settings
    host: str = '0.0.0.0'
    port: int = 8080
    root_path: str = ''

    # redis settings
    redis_url: RedisDsn = 'redis://localhost:6379/0'

    # elastic settings
    elastic_url: str = 'http://localhost:9200/'
    elastic_index: str = 'oas'

    # set to 1 to enable development mode
    # (hot reload code on changes)
    oas_dev: bool = False

    # config parsing
    class Config:
        env_file = '.env'


config = Settings()
