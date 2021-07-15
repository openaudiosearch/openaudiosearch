from pydantic import BaseModel
from enum import Enum
from typing import List, Any

class Engine(str, Enum):
    vosk = "vosk"
    pytorch = "pytorch"


class AsrArgs(BaseModel):
    media_file: str


class AsrOpts(BaseModel):
    engine: Engine
    language: str = 'de'


class AsrResult(BaseModel):
    text: str
    parts: List[Any] = []
