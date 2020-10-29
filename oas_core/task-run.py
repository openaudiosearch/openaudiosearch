#!/usr/bin/env python3

"""
Run a task immediately (without queue)
"""

import logging
import sys
import typing as T

from app.core.cli_util import run_task_cli
from app.worker import worker


def runner(task_name, args, opts):
    print(f'Running task "{task_name}" with args: {args}')
    worker.queue_job(task_name, args, opts)
    worker.run()


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    run_task_cli(runner, description=__doc__)
