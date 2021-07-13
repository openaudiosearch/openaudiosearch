from celery import Celery
from app.config import config

app = Celery('tasks', broker=config.redis_url, backend=config.redis_url, result_extended=True)

app.conf.update(
    #  result_backend=None,
    task_ignore_result=True,
    task_routes=(
        [("*", {"queue": "celery"})],
    ),
)
