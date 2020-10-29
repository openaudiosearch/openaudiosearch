from pydantic_cli import SubParser, run_sp_and_exit, FailedExecutionException

from app.tasks.models import TASKS

VERSION = '0.1.0'


def __to_runner(task_name, args_model, opts_model, runner):
    def run_task(cli_opts):
        args = args_model(**cli_opts.dict())
        opts = opts_model(**cli_opts.dict())
        runner(task_name, args, opts)
    return run_task


def tasks_to_subparser(runner):
    spec = {}
    for (task_name, params) in TASKS.items():
        class CliArgs(params[0], params[1]):
            pass
        spec[task_name] = SubParser(CliArgs, __to_runner(
            task_name, params[0], params[1], runner), f'{task_name} description (TODO)')
    return spec


def exception_handler(ex) -> int:
    print(ex)
    # logging.error(ex, exc_info=logging.getLevelName() == logging.DEBUG)
    return 1


def run_task_cli(runner, description, version=VERSION):
    run_sp_and_exit(tasks_to_subparser(runner),
                    description, version,
                    exception_handler=exception_handler)
