from celery import Celery
from app.config import config

app = Celery('tasks', broker=config.redis_url, backend=config.redis_url, result_extended=True)
