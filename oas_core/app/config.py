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

    # set to 1 to enable development mode
    # (hot reload code on changes)
    oas_dev: bool = False

    # config parsing
    class Config:
        env_file = '.env'


config = Settings()
