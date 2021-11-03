from pydantic import BaseModel
from typing import Any, List, Optional
from enum import Enum


class Engine(str, Enum):
    vosk = "vosk"
    pytorch = "pytorch"
    foo = 'foo'


class DownloadArgs(BaseModel):
    media_url: str


class DownloadOpts(BaseModel):
    refresh = False


class DownloadResult(BaseModel):
    source_url: Optional[str] = None
    file_path: str


class AsrOpts(BaseModel):
    engine: Engine
    language: str = 'de'
    samplerate = 16000


class AsrResult(BaseModel):
    text: str
    parts: List[Any] = []


class NlpOpts(BaseModel):
    pipeline: str = 'ner,pos,lemma,missed'


class NlpResult(BaseModel):
    result: dict


class TranscribeArgs(BaseModel):
    media_url: str
    doc_id: Optional[str] = None


class TranscribeOpts( AsrOpts, NlpOpts):
    pass


class ElasticIndexArgs(BaseModel):
    asr_result: AsrResult
    path_to_audio: str


class ElasticIndexOpts(BaseModel):
    pass


class ElasticSearchArgs(BaseModel):
    search_term: str


class ElasticSearchOpts(BaseModel):
    pass


    # this is used by the CLI
TASKS = {
    'transcribe': (TranscribeArgs, TranscribeOpts),
    'download': (DownloadArgs, DownloadOpts),
}
