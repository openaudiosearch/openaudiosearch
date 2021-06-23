#!/usr/bin/env python3

import argparse
import os
import sys
import time

from pydantic import BaseModel
from enum import Enum
from typing import List, Any

from .models import Engine, AsrArgs, AsrOpts, AsrResult

from celery import Celery
from celery.bin.celery import main as _main
#  broker_address = os.environ.get("AMQP_ADDR", "amqp://127.0.0.1:5672/oas")
broker_address = os.environ.get("REDIS_ADDRESS", "redis://127.0.0.1:6379/0")

app = Celery("celery", broker=broker_address)

app.conf.update(
    result_backend=None,
    task_ignore_result=True,
    task_routes=(
        [("*", {"queue": "celery"})],
    ),
)

@app.task(name="add")
def add_numbers(a, b):
    print("add_numbers", a, b)
    time.sleep(2)
    return a + b

@app.task(name="asr")
def asr(args: AsrArgs, opts: AsrOpts) -> AsrResult:
    print("args", args)
    print("opts", opts)
    return AsrResult(text="hello from python", parts=[])
