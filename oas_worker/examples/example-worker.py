import time
import sys
sys.path.insert(0, "..")

from app.worker import worker
from app.bin import run

## this imports jobs defined in the examples folder
## it is meant for in-development jobs and examples
# that are not yet ready to run on the production system

import app.examples.example


if __name__ == '__main__':
    run(worker)

