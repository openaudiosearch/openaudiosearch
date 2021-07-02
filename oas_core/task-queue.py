#!/usr/bin/env python3

"""
Add a task to the work queue
"""

import argparse
from app.server.jobs import Jobs

jobs = Jobs()

def runner(task_name, args):
    if task_name == "transcribe":
        print(f'Queuing task "{task_name}" with args: {args}')
        result = jobs.create_transcript_job(args.media_url)
        print(f'job id: {result.id}')


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Add a task to the work queue')
    parser.add_argument("task_name", type=str, help="queue task_name, e.g. transcribe")
    parser.add_argument("media_url", type=str, help="URL to transcribe, if task_name = transcribe")
    args = parser.parse_args()
    
    runner(args.task_name, args)
