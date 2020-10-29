#!/usr/bin/env python3

import sys
import typing as T
from pydantic_cli import run_and_exit, to_runner

from app.tasks.models import (
    DownloadArgs,
    TranscribeArgs,
    TranscribeOpts
)
# from app.tasks.models import *

from pydantic_cli import run_sp_and_exit, SubParser


class TranscribeArgsOpts(TranscribeOpts, TranscribeArgs):
    pass


def transcribe_runner(opts: TranscribeOpts):
    print(f'TRANSCRIBE {opts}')


def download_runner(opts: DownloadArgs):
    print(f'DOWNLOAD {opts}')


def to_subparser_example():
    return {
        'transcribe': SubParser(TranscribeOpts, transcribe_runner, "Run full transcribe pipeline"),
        'download': SubParser(DownloadArgs, download_runner, "Run download task")}


if __name__ == "__main__":
    run_sp_and_exit(to_subparser_example(),
                    description=__doc__, version='0.1.0')
