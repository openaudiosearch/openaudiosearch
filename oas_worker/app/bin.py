import argparse
import logging
import time
import os

from app.config import config
from app.logging import log
from app.worker import worker, DASHBOARD_PORT

def run_with_args(worker, args):
    log.info(f"start OAS worker (pid {os.getpid()})")
    log.info(f"registered jobs: {list(worker.jobs)}")
    try:
        if args.single is not None:
            worker.single(args.single)
        else:
            worker.start()
    except KeyboardInterrupt:
        log.warning("Received Ctrl-C, shutting down...")
        worker.stop()

    except BaseException as e:
        log.error(f"Error: {e}", exception=e)
        worker.stop()
        exit(1)

def run(worker):
    description = "Open Audio Search worker"
    epilog = f"Registered jobs: {list(worker.jobs)}"
    parser = argparse.ArgumentParser(description=description, epilog=epilog)
    parser.add_argument('--single', type=str, help='Run a single job of this typ', metavar="JOB")
    #  parser.add_argument('--dashboard', action='store_true', help='Enable the job dashboard on port {DASHBOARD_PORT}')
    args = parser.parse_args()
    run_with_args(worker, args)

