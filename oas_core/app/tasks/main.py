import os
from urllib.parse import urlparse
from base64 import b32encode

import requests
from pydub import AudioSegment
from hashlib import sha256
from mimetypes import guess_extension

from app.config import config
from app.worker import worker
from app.core.job import Task
from app.tasks.models import *
from app.core.util import download_file, pretty_bytes
from app.tasks.spacy_pipe import SpacyPipe
from app.tasks.transcribe_vosk import transcribe_vosk

import app.tasks.download_models


def url_to_path(url: str) -> str:
    parsed_url = urlparse(url)

    url_hash = sha256(url.encode('utf-8')).digest()
    url_hash = b32encode(url_hash)
    url_hash = url_hash.lower().decode('utf-8')

    target_name = f'{parsed_url.netloc}/{url_hash[:2]}/{url_hash[2:]}'
    return target_name


@worker.task('download', description='Download a media file', result=PrepareArgs)
def download(task: Task, args: DownloadArgs, opts: DownloadOpts) -> PrepareArgs:
    # todo: maybe cache downloads globally by url hash
    # instead of locally per job
    # target_filename = task.file_path('download/' + url_hash, root=True)
    #  if os.path.exists(destination_file):

    url = args.media_url
    target_name = url_to_path(url)
    target_path = task.file_path(
        f'download/{target_name}', root=True)
    temp_path = task.file_path('download.tmp')
    # chunk size to write
    chunk_size = 1024*64

    with requests.get(url, stream=True) as res:
        res.raise_for_status()

        headers = res.headers
        extension = guess_extension(
            headers['content-type'].partition(';')[0].strip())
        target_path += extension
        total_size = int(headers.get('content-length', 0))

        # check if the file exists and return early
        if os.path.isfile(target_path) and not opts.refresh:
            task.log(
                f'File exists, skipping download of {args.media_url} to {target_path} ({pretty_bytes(total_size)})')
            return PrepareArgs(file_path=target_path)

        task.log(
            f'Downloading {url} to {target_path} ({pretty_bytes(total_size)})')

        total_size = int(res.headers.get('content-length', 0))
        download_size = 0
        with open(temp_path, 'wb') as f:
            for chunk in res.iter_content(chunk_size=chunk_size):
                download_size += len(chunk)
                # If you have chunk encoded response uncomment if
                # and set chunk_size parameter to None.
                # if chunk:
                f.write(chunk)
                f.flush()
                percent = download_size / total_size
                task.report_progress(percent)

        os.rename(temp_path, target_path)
        return PrepareArgs(file_path=target_path)


@worker.task('prepare')
def prepare(task: Task, args: PrepareArgs, opts: PrepareOpts) -> AsrArgs:
    dst = task.file_path('processed.wav')
    sound = AudioSegment.from_file(args.file_path)
    sound.set_frame_rate(opts.samplerate)
    sound.set_channels(1)
    sound.export(dst, format="wav")
    return AsrArgs(file_path=dst)


@worker.task('asr')
def asr(task: Task, args: AsrArgs, opts: AsrOpts) -> AsrResult:
    model_path = os.path.join(config.model_path, config.model)
    if opts.engine == "vosk":
        result = transcribe_vosk(args.file_path, model_path)
        print(f'RESULT: {result}')
        return AsrResult(text=str(result))
    elif opts.engine == "deepspeech":
        raise NotImplementedError("ASR using deepspeech is not available yet")
    elif opts.engine == "torch":
        raise NotImplementedError("ASR using torch is not available yet")
    else:
        raise RuntimeError("ASR engine not specified")
    return AsrResult(text='')


@worker.task('nlp')
def nlp(task: Task, args: AsrResult, opts: NlpOpts) -> NlpResult:
    spacy = SpacyPipe(opts.pipeline)
    res = spacy.run(args.text)
    return NlpResult(result=res)


@worker.task('transcribe')
def transcribe(task: Task, args: TranscribeArgs, opts: TranscribeOpts):

    nlp_opts = NlpOpts(pipeline='ner')

    job = task.job
    job.add_task(download, opts=DownloadOpts())
    job.add_task(prepare, opts=opts)
    job.add_task(asr, opts=opts)
    job.add_task(nlp, opts=nlp_opts)

    return DownloadArgs(media_url=args.media_url)
