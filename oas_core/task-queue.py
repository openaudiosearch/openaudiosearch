#!/usr/bin/env python3

"""
Add a task to the work queue
"""

import logging
import sys
import typing as T

from app.core.cli_util import run_task_cli
from app.server.jobs import jobs


def runner(task_name, args, opts):
    print(f'Queuing task "{task_name}" with args: {args}')
    id = jobs.queue_job(task_name, args, opts)
    print(f'job id: {id}')


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    run_task_cli(runner, description=__doc__)
