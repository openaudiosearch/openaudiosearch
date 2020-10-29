from pydantic import BaseModel

# tasks = [
#   ('download', DownloadArgs, None),
#   ('transcribe', TranscribeArgs, TranscribeOpts),
# ]

class DownloadArgs(BaseModel):
    media_url: str
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

class TranscribeArgs(BaseModel):
    media_url: str

class TranscribeOpts(PrepareOpts, AsrOpts):
    foo = 'bar'
    # prepare: PrepareOpts = PrepareOpts()
    # asr: AsrOpts