import os
import requests
import time
import json
import httpx
import subprocess
import datetime
from pathlib import Path
from mimetypes import guess_extension

from app.config import config
from app.util import pretty_bytes, ensure_dir, url_to_path, find_in_dict
from app.logging import log
from app.worker import worker

from app.jobs.spacy_pipe import SpacyPipe
from app.jobs.transcribe_vosk import transcribe_vosk

def local_path_with_dir(path):
    path = config.local_path(path)
    os.makedirs(Path(path).parent, exist_ok=True)
    return path

def download(url, refetch=False, limit_bytes=None):
    url_as_path = url_to_path(url)
    target_path = local_path_with_dir(f"download/{url_as_path}")
    temp_path = target_path + ".tmp"
    # chunk size to write
    chunk_size = 1024 * 64
    with requests.get(url, stream=True) as res:
        res.raise_for_status()

        headers = res.headers
        extension = guess_extension(headers["content-type"].partition(";")[0].strip())
        if extension is None:
            extension = ".bin"
        target_path += extension
        total_size = int(headers.get("content-length", 0))

        if os.path.isfile(target_path) and not refetch:
            log.info(
                f"File exists, skipping download of {url} to {target_path} ({pretty_bytes(total_size)})"
            )
            return target_path

        log.info(f"Downloading {url} to {target_path} ({pretty_bytes(total_size)})")

        total_size = int(res.headers.get("content-length", 0))
        download_size = 0
        with open(temp_path, "wb") as f:
            for chunk in res.iter_content(chunk_size=chunk_size):
                download_size += len(chunk)
                f.write(chunk)
                f.flush()
                if limit_bytes is not None and download_size > limit_bytes:
                    break

        os.rename(temp_path, target_path)
        return target_path


@worker.job(name="asr")
def asr(ctx, args):
    media_id = args["media_id"]
    engine = args.get("engine") or "vosk"
    samplerate = args.get("samplerate") or 16000

    if engine != "vosk":
        raise NotImplementedError(f"Speech recognition engine `{engine}` is not implemented")

    model_base_path = config.model_path
    model_path = os.path.join(model_base_path, config.model)

    # fetch media record
    media = ctx.get("/media/" + media_id)
    guid = media["$meta"]["guid"]
    url = media["contentUrl"]

    # download media file
    downloaded_path = download(url)

    # convert to wav
    workdir = ctx.workdir()
    os.makedirs(workdir, exist_ok=True)
    temp_wav = os.path.join(workdir, "processed.wav")

    subprocess.call(
        [
            "ffmpeg",
            "-i",
            downloaded_path,
            "-hide_banner",
            "-loglevel",
            "error",
            "-ar",
            str(samplerate),
            "-ac",
            "1",
            temp_wav,
        ],
        stdout=subprocess.PIPE,
    )

    # transcribe with vosk
    start = time.time()
    result = transcribe_vosk(media_id, temp_wav, model_path)
    duration = time.time() - start

    try:
        os.remove(temp_wav)
        os.rmdir(workdir)
    except BaseException as e:
        ctx.log.warning("Failed to delete job workdir `{dir}`", exception=e)

    patch = [
        {"op": "replace", "path": "/transcript", "value": result},
    ]
    return {
        "patches": {
            guid: patch
        },
        "meta": {
            "engine": "vosk",
            "model": config.model,
            "transcribe_duration": str(duration)
        }
    }


@worker.job(name="nlp")
def nlp(ctx, args):
    post_id = args["post_id"]
    pipeline = "ner"
    post = ctx.get(f"/post/{post_id}")
    guid = post["$meta"]["guid"]
    fields = ["headline", "description", "transcript.text"]
    values = filter(None.__ne__, map(lambda f: find_in_dict(post, f), fields))
    text = "\n".join(values)
    spacy = SpacyPipe(pipeline)
    res = spacy.run(text)
    patch = [
        {"op": "replace", "path": "/nlp", "value": res},
    ]
    patches = { guid: patch }
    return {
        "patches": patches
    }


# Debug test job to skip ASR but return valid results.
@worker.job(name="asr_mock")
def asr_mock(ctx, args):
    media_id = args["media_id"]
    media = ctx.get("/media/" + media_id)
    guid = media["$meta"]["guid"]
    result = {
        "text": "foo bar baz",
        "parts": [
            { "start": 0.2, "end": 1.0, "conf": 1.0, "word": "foo" },
            { "start": 0.2, "end": 1.0, "conf": 1.0, "word": "foo" },
            { "start": 0.2, "end": 1.0, "conf": 1.0, "word": "foo" },
        ]
    }
    patch = [
        {"op": "replace", "path": "/transcript", "value": result},
    ]
    patches = { guid: patch }
    meta = { "mock": "yes" }
    return { "patches": patches, "meta": meta }

