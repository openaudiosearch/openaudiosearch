from .models import Engine, AsrArgs, AsrOpts, AsrResult
import time

def add_numbers(a, b):
    print("add_numbers", a, b)
    time.sleep(2)
    return a + b

def asr(args: AsrArgs, opts: AsrOpts) -> AsrResult:
    print("args", args)
    print("opts", opts)
    time.sleep(1)
    return AsrResult(text="hello from python", parts=[])
