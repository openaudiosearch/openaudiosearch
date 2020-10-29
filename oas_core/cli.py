import sys
import typing as T
from pydantic_cli import run_and_exit, to_runner

from app.tasks.models import *


# def example_runner(opts: TranscribeOpts) -> int:
#     print(f"Mock example running with options {opts}")
#     return 0

# if __name__ == '__main__':
#     # to_runner will return a function that takes the args list to run and 
#     # will return an integer exit code
#     sys.exit(to_runner(TranscribeOpts, example_runner, version='0.1.0')(sys.argv[1:]))



# from pydantic import BaseModel, AnyUrl
# from pydantic_cli.examples import ExampleConfigDefaults
from pydantic_cli import run_sp_and_exit, SubParser


# class AlphaOptions(BaseModel):

#     class Config(ExampleConfigDefaults):
#         CLI_EXTRA_OPTIONS = {'max_records': ('-m', '--max-records')}

#     input_file: str
#     max_records: int = 10


# class BetaOptions(BaseModel):

#     class Config(ExampleConfigDefaults):
#         CLI_EXTRA_OPTIONS = {'url': ('-u', '--url'),
#                              'num_retries': ('-n', '--num-retries')}

#     url: AnyUrl
#     num_retries: int = 3


# def printer_runner(opts: T.Any):
#     print(f"Mock example running with {opts}")
#     return 0


# def to_runner(sx):
#     def example_runner(opts) -> int:
#         print(f"Mock {sx} example running with {opts}")
#         return 0
#     return example_runner

class TranscribeArgsOpts(TranscribeOpts, TranscribeArgs):
  pass

def transcribe_runner(opts: TranscribeArgsOpts):
  print(f'TRANSCRIBE {opts}')

def download_runner(opts: DownloadArgs):
  print(f'DOWNLOAD {opts}')

def to_subparser_example():
    return {
        'transcribe': SubParser(TranscribeArgsOpts, transcribe_runner, "Run full transcribe pipeline"),
        'download': SubParser(DownloadArgs, download_runner, "Run download task")}


if __name__ == "__main__":
    run_sp_and_exit(to_subparser_example(), description=__doc__, version='0.1.0')