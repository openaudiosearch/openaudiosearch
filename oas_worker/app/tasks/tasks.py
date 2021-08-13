import os
import json
import httpx
import subprocess
from pathlib import Path
from urllib.parse import urlparse
from base64 import b32encode

import requests
from hashlib import sha256
from mimetypes import guess_extension
from celery import Celery, chain
from celery.utils.log import get_task_logger
from app.tasks.models import *

from app.config import config
from app.util import pretty_bytes
from app.celery import app

from app.tasks.spacy_pipe import SpacyPipe
from app.tasks.transcribe_vosk import transcribe_vosk

logger = get_task_logger(__name__)
cache_path = os.path.join(config.storage_path, 'cache')

def file_path(filename):
    path = os.path.join(config.storage_path, filename)
    ensure_dir(path)
    return path

def ensure_dir(path: str):
    path = Path(path)
    os.makedirs(path.parent, exist_ok=True)

def url_to_path(url: str) -> str:
    parsed_url = urlparse(url)

    url_hash = sha256(url.encode('utf-8')).digest()
    url_hash = b32encode(url_hash)
    url_hash = url_hash.lower().decode('utf-8')

    target_name = f'{parsed_url.netloc}/{url_hash[:2]}/{url_hash[2:]}'
    return target_name

@app.task(name="transcribe")
def transcribe(args: dict, opts: dict):
    print("start transcribe task with args", args, "opts", opts)
    media_url = args['media_url']
    media_id = args['media_id']
    nlp_opts = {'pipeline': 'ner'}
    result = chain(
        download.s({'media_url': media_url, 'media_id': media_id }),
        prepare.s({'samplerate': 16000}),
        asr.s({'engine': 'vosk'}),
        nlp.s(nlp_opts),
        save_media.s()
        )()
    return result
    

@app.task(name="download")
def download(opts):
    args = {"opts": opts}
    # todo: maybe cache downloads globally by url hash
    # instead of locally per job
    # target_filename = task.file_path('download/' + url_hash, root=True)
    #  if os.path.exists(destination_file):

    url = opts['media_url']
    target_name = url_to_path(url)
    target_path = file_path(
        f'download/{target_name}')
    # target_path = task.file_path(
        # f'download/{target_name}', root=True)
    # temp_path = file_path('download.tmp')
    temp_path = file_path(f'task/download/{download.request.id}/download.tmp')
    # chunk size to write
    chunk_size = 1024*64

    with requests.get(url, stream=True) as res:
        res.raise_for_status()

        headers = res.headers
        extension = guess_extension(
            headers['content-type'].partition(';')[0].strip())
        if extension is None:
            extension = "bin"
        target_path += extension
        total_size = int(headers.get('content-length', 0))

        # check if the file exists and return early
        # if os.path.isfile(target_path) and not opts.refresh:
        if os.path.isfile(target_path):
            logger.info(
                f'File exists, skipping download of {url} to {target_path} ({pretty_bytes(total_size)})')
            # return DownloadResult(file_path=target_path, source_url=url)
            args["download"] = {"file_path": target_path, "source_url": url}
            return args

        logger.info(
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

        os.rename(temp_path, target_path)
        # return DownloadResult(file_path=target_path, source_url=url)
        args['download'] = {"file_path": target_path, "source_url": url}
        return args

@app.task(name="prepare")
def prepare(args, opts):
    samplerate = opts['samplerate']
    dst = file_path(f'task/prepare/{prepare.request.id}/processed.wav')
    # TODO: Find out why this pydub segment does not work.
    # sound = AudioSegment.from_file(args.file_path)
    # sound.set_frame_rate(opts.samplerate)
    # sound.set_channels(1)
    # sound.export(dst, format="wav")
    subprocess.call(['ffmpeg', '-i',
                     args["download"]["file_path"],
                     '-hide_banner', '-loglevel', 'error',
                     '-ar', str(samplerate), '-ac', '1', dst],
                    stdout=subprocess.PIPE)
    args['prepare'] = {'file_path': dst}
    return args

@app.task(name="asr")
def asr(args, opts):
    engine = opts['engine']

    model_base_path = config.model_path or os.path.join(
        config.storage_path, 'models')
    model_path = os.path.join(model_base_path, config.model)
    if engine == "vosk":
        result = transcribe_vosk(args["prepare"]["file_path"], model_path)
        args["asr"] = result
        return args
    elif engine == "deepspeech":
        raise NotImplementedError("ASR using deepspeech is not available yet")
    elif engine == "torch":
        raise NotImplementedError("ASR using torch is not available yet")
    else:
        raise RuntimeError("ASR engine not specified")

@app.task(name="nlp")
def nlp(args, opts):
    spacy = SpacyPipe(opts['pipeline'])
    res = spacy.run(args['asr']['text'])
    args['nlp'] = res
    return args


@app.task(name="save_media")
def save_media(args):
    media_id = args['opts']['media_id']
    url = f"{config.oas_url}/media/{media_id}"
    data = {
        "transcript": args['asr'],
        "nlp": args['nlp']
    };
    res = httpx.patch(url, json=data)
    res = res.json()
    print("res", res)
    return res

@app.task(name="nlp2")
def nlp2(args, opts):
    post = args
    print("START NLP", post)
    url = f"{config.oas_url}/media/{media_id}"

@app.task(name="download2")
def download2(args, opts):
    media = args
    print("START DOWNLOAD", media)
