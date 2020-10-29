#!/usr/bin/env python3

"""
Get result from a task
"""

import logging
import sys
import typing as T
import json
from argparse import ArgumentParser

from app.server.jobs import jobs

if __name__ == "__main__":
    parser = ArgumentParser(description='Process some integers.')
    parser.add_argument('id', metavar='ID', type=str,
                        help='A job ID')

    args = parser.parse_args()
    result = jobs.get_result(args.id)
    print(json.dumps(result, indent=4))
