# pylint: disable=wrong-import-position
from app.core.job import Worker

# create a worker
worker = Worker()

# register all tasks
# the noqa tells autopep8 to not move the import up
import app.tasks.main  # noqa
