import time

from app.worker import worker
from app.bin import run

import app.jobs.jobs
import app.jobs.recasepunc
from app.jobs.recasepunc import WordpieceTokenizer 
if __name__ == '__main__':
    run(worker)

