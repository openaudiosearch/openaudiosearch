import time

from app.worker import worker
from app.bin import run

#import app.jobs.jobs

if __name__ == '__main__':
    run(worker)

