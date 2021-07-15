import os
from faktory import Worker

from app.models import Engine, AsrArgs, AsrOpts, AsrResult
from app.faktory_tasks import add_numbers, asr

faktory_url = os.environ.get("FAKTORY_URL", "tcp://localhost:7419")

w = Worker(faktory_url,
           queues=['default'],
           concurrency=1)
w.register('add', add_numbers)
w.register('asr', asr)

w.run()
