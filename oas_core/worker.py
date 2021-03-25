#!/usr/bin/env python3

import logging

from app.core.job import Worker
from app.tasks.models import TranscribeOpts, TranscribeArgs
from app.worker import worker

from app.core.job import Client
from app.config import config

if __name__ == '__main__':
    logging.basicConfig(level=logging.INFO)
    client = Client(redis_url=config.redis_url)

    while True:
        task = client.dequeue_task()
        if task:
            logging.info(f'START task {task.job_id}: {task.task_name}')
            worker.queue_job(task.task_name, task.args,
                             task.opts, id=task.job_id)
            worker.run()
            logging.info(f'DONE task {task.job_id}: {task.task_name}')
