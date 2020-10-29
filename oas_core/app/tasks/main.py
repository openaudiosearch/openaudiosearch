from pydub import AudioSegment
import os
from hashlib import sha256
from mimetypes import guess_extension

from app.worker import worker
from app.core.job import Task
from app.tasks.models import *
from app.core.util import download_file, pretty_bytes


@worker.task('download', result=PrepareArgs)
def download(task: Task, args: DownloadArgs, opts: DownloadOpts) -> PrepareArgs:
    # todo: maybe cache downloads globally by url hash
    # instead of locally per job
    #url_hash = sha256(args.media_url).hexdigest()
    #  if os.path.exists(destination_file):

    temp_filename = task.file_path('download.tmp')
    target_filename = task.file_path('media')
    total_size = 0

    def on_headers(headers):
        nonlocal target_filename, task
        extension = guess_extension(
            headers['content-type'].partition(';')[0].strip())
        target_filename += extension
        total_size = int(headers.get('content-length', 0))
        task.log(
            f'Downloading {args.media_url} to {target_filename} ({pretty_bytes(total_size)})')

    def on_progress(percent):
        nonlocal task
        task.report_progress(percent)

    download_file(args.media_url, temp_filename,
                  on_headers=on_headers, on_progress=on_progress)
    os.rename(temp_filename, target_filename)
    return PrepareArgs(file_path=target_filename)


@worker.task('prepare')
def prepare(task: Task, args: PrepareArgs, opts: PrepareOpts) -> AsrArgs:
    dst = task.file_path('processed.wav')
    sound = AudioSegment.from_mp3(args.file_path)
    sound.set_frame_rate(opts.samplerate)
    sound.set_channels(1)
    sound.export(dst, format="wav")
    return AsrArgs(file_path=dst)


@worker.task('asr')
def asr(task: Task, args: AsrArgs, opts: AsrOpts) -> AsrResult:
    text = f'hei this is what them talk on {args.file_path}, at least that is what {opts.engine} thinks'
    return AsrResult(text=text)


@worker.task('transcribe')
def transcribe(task: Task, args: TranscribeArgs, opts: TranscribeOpts):
    job = task.job
    job.add_task(download)
    job.add_task(prepare, opts=opts)
    job.add_task(asr, opts=opts)
    return DownloadArgs(media_url=args.media_url)
