import time

from app.worker import worker
from app.bin import run

import app.jobs.jobs

# Load the class that the recasepunc worker needs for unpicling right here
# in the main executable.
# This is basically a bug in pytorch: It needs the class on the same root
# module where it was saved. As we use a model that was create from
# a foreign script in __main__ we need to import the class here.
# See https://stackoverflow.com/questions/55488795 for details.
from recasepunc import WordpieceTokenizer

# again run some global pytorch init stuff
import torch
torch.set_num_threads(1)
# torch.multiprocessing.set_start_method('spawn')
torch.device("cpu")


if __name__ == '__main__':
    run(worker)

