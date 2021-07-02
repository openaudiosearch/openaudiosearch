from pydantic import BaseModel
from pydantic_cli import SubParser, run_sp_and_exit, FailedExecutionException

VERSION = '0.1.0'


class EmptyArgs(BaseModel):
    pass


class EmptyOpts(BaseModel):
    pass


def __to_runner(task_name, args_model, opts_model, runner):
    def run_task(cli_opts):
        args = args_model(**cli_opts.dict())
        opts = opts_model(**cli_opts.dict())
        runner(task_name, args, opts)
    return run_task


def tasks_to_subparser(runner):
    spec = {}
    for (task_name, wrapped_task_fn) in worker.registered_tasks.items():
        (args_model, opts_model) = wrapped_task_fn.get_typed_signature()
        if not issubclass(args_model, BaseModel):
            args_model = EmptyArgs
        if not issubclass(opts_model, BaseModel):
            opts_model = EmptyOpts

        class CliArgs(args_model, opts_model):
            pass

        spec[task_name] = SubParser(CliArgs, __to_runner(
            task_name, args_model, opts_model, runner), wrapped_task_fn.description)
    return spec


def exception_handler(ex) -> int:
    print(ex)
    # logging.error(ex, exc_info=logging.getLevelName() == logging.DEBUG)
    return 1


def run_task_cli(runner, description, version=VERSION):
    run_sp_and_exit(tasks_to_subparser(runner),
                    description, version,
                    exception_handler=exception_handler)
