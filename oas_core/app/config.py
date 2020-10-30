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
    redis: RedisDsn = 'redis://user:pass@localhost:6379/1'
    log_level: str = 'info'

    # server settings
    host: str = '0.0.0.0'
    port: int = 8080
    root_path: str = ''

    # config parsing
    class Config:
        env_file = '.env'


config = Settings()
