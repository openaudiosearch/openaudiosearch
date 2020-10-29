from pydantic import BaseModel


class DownloadArgs(BaseModel):
    media_url: str


class DownloadOpts(BaseModel):
    refresh = False


class PrepareArgs(BaseModel):
    file_path: str


class PrepareOpts(BaseModel):
    samplerate = 16000


class AsrArgs(BaseModel):
    file_path: str


class AsrOpts(BaseModel):
    engine: str
    language: str = 'de'


class AsrResult(BaseModel):
    text: str


class NlpArgs(BaseModel):
    pipeline: str
    text: str


class NlpResult(BaseModel):
    result: dict


class TranscribeArgs(BaseModel):
    media_url: str


class TranscribeOpts(PrepareOpts, AsrOpts):
    samplerate = 16000
    engine: str
    language: str = 'de'



# this is used by the CLI
TASKS = {
    'transcribe': (TranscribeArgs, TranscribeOpts),
    'download': (DownloadArgs, DownloadOpts),
}
