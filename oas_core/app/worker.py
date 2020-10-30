# pylint: disable=wrong-import-position
import os
from app.core.job import Worker, Task
from app.config import config

# create a worker
cache_path = os.path.join(config.storage_path, 'cache')
worker = Worker(cache_path=cache_path)


def task(name=None, result=None):
    worker.task(name=name, result=result)


# register all tasks
# the noqa tells autopep8 to not move the import up
import app.tasks.main  # noqa
