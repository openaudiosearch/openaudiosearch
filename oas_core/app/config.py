from pydantic import (
    BaseModel,
    BaseSettings,
    PyObject,
    RedisDsn,
    Field
)


class Settings(BaseSettings):
    # general settings
    storage_path: str = '/tmp/oas'
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

    es_host: str = '0.0.0.0'
    es_port: int = 9200
    es_index: str = 'oas'

    # set to 1 to enable development mode
    # (hot reload code on changes)
    oas_dev: bool = False

    # config parsing
    class Config:
        env_file = '.env'


config = Settings()
