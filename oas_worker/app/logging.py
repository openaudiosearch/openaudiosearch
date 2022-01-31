import os
import logging
from pprint import pformat
import sys
from loguru import logger

from app.config import config

CLI_FORMAT = "{time:YYYY-MM-DDTHH:mm:ss.SS} <level>{level}</level> [{extra[name]}] {message}"
FILE_FORMAT = "{time:YYYY-MM-DDTHH:mm:ss.SS} pid:{process} level:{level} worker:{extra[name]} {message}"

level = config.log_level.upper()
diagnose = False
if level == "DEBUG" or level == "TRACE":
    diagnose = True
log = logger

def enrich_record(record: dict) -> dict:
    if record["extra"].get("name") is None:
        record["extra"]["name"] = record["name"].replace("__", "")
    if record["extra"].get("id") is not None:
        id = record["extra"]["id"]
        record["extra"]["name"] += f"-{id}"
    return record

def cli_formatter(record: dict) -> str:
    record = enrich_record(record)
    fmt = CLI_FORMAT
    fmt += "\n"
    if diagnose:
        fmt = fmt_exception(fmt, record)
    return fmt

def file_formatter(record: dict) -> str:
    record = enrich_record(record)
    fmt = FILE_FORMAT
    fmt += "\n"
    return fmt

def fmt_exception(fmt, record):
    if record["exception"] is not None:
        fmt += "{exception}\n"
    return fmt

def setup_logging():
    log_path = config.log_file
    logger.remove()
    logger.add(sys.stderr,
            colorize=True,
            backtrace=False,
            #  diagnose=False,
            #  backtrace=diagnose,
            diagnose=diagnose,
            format=cli_formatter,
            level=level)

    logger.add(log_path,
            rotation="50 MB",
            diagnose=False,
            format=file_formatter,
            level="INFO")

    #  logging.basicConfig(handlers=[InterceptHandler()], level=0)


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




setup_logging()
