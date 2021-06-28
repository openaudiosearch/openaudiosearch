#!/usr/bin/env python3

"""
Get result from a task
"""

from argparse import ArgumentParser

from app.server.jobs import Jobs

if __name__ == "__main__":
    jobs = Jobs()
    parser = ArgumentParser(description='Process some integers.')
    parser.add_argument('id', metavar='ID', type=str,
                        help='A job ID')

    args = parser.parse_args()
    result = jobs.get_job(args.id)
    print(result)
