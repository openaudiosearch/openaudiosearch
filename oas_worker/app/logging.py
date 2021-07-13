import os
import logging
from pprint import pformat
import sys
from loguru import logger
from app.config import config

level = os.environ.get('LOG', 'INFO').upper()
diagnose = False
if level == "DEBUG" or level == "TRACE":
    diagnose = True

FORMAT = "{time:YYYY-MM-DDTHH:mm:ss.SS} <level>{level}</level>] {message}"

def formatter(record):
    fmt = f'{FORMAT}\n'
    fmt = fmt_exception(fmt, record)
    return fmt

def fmt_exception(fmt, record):
    if record["exception"] is not None:
        if diagnose:
            fmt += "{exception}\n"
    return fmt

def setup_logging():
    log_path = os.path.join(config.storage_path, 'oas.log')
    logger.remove()
    logger.add(sys.stderr,
            colorize=True,
            backtrace=False,
            diagnose=False,
            #  backtrace=diagnose,
            #  diagnose=diagnose,
            level=level,
            format=formatter)

    logger.add(log_path,
            rotation="50 MB",
            format="[{time:YYYY-MM-DD HH:mm:ss.SS} {level}] {message}",
            level="INFO")


class InterceptHandler(logging.Handler):
    """
    Default handler from examples in loguru documentaion.
    See https://loguru.readthedocs.io/en/stable/overview.html#entirely-compatible-with-standard-logging
    """

    def emit(self, record: logging.LogRecord):
        # Get corresponding Loguru level if it exists
        try:
            level = logger.level(record.levelname).name
        except ValueError:
            level = record.levelno

        # Find caller from where originated the logged message
        frame, depth = logging.currentframe(), 2
        while frame.f_code.co_filename == logging.__file__:
            frame = frame.f_back
            depth += 1

        logger.opt(depth=depth, exception=record.exc_info).log(
            level, record.getMessage()
        )


def format_record(record: dict) -> str:
    """
    Custom format for loguru loggers.
    Uses pformat for log any data like request/response body during debug.
    Works with logging if loguru handler it.
    """

    fmt = FORMAT
    if record["extra"].get("payload") is not None:
        record["extra"]["payload"] = pformat(
            record["extra"]["payload"], indent=4, compact=True, width=88
        )
        fmt += "\n{extra[payload]}"
    fmt += "\n"
    fmt = fmt_exception(fmt, record)
    return fmt


def setup_uvicorn_logging():
    """
    Replaces logging handlers with a handler for using the custom handler.
    """

    # disable handlers for specific uvicorn loggers
    # to redirect their output to the default uvicorn logger
    # works with uvicorn==0.11.6
    loggers = (
        logging.getLogger(name)
        for name in logging.root.manager.loggerDict
        if name.startswith("uvicorn.")
    )
    for uvicorn_logger in loggers:
        uvicorn_logger.handlers = []

    # change handler for default uvicorn logger
    intercept_handler = InterceptHandler()
    logging.getLogger("uvicorn").handlers = [intercept_handler]

    # set logs output, level and format
    logger.configure(
        handlers=[{"sink": sys.stdout, "level": logging.DEBUG, "format": format_record}]
    )


setup_logging()
setup_uvicorn_logging()
