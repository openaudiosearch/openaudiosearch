import os
from celery import Celery
from celery.utils.log import get_task_logger
from app.tasks.models import *

app = Celery('tasks', broker='redis://localhost')

logger = get_task_logger(__name__)

@app.task
def add(x, y):
    return x + y

@app.task
def download(media_url) -> DownloadResult:
    # todo: maybe cache downloads globally by url hash
    # instead of locally per job
    # target_filename = task.file_path('download/' + url_hash, root=True)
    #  if os.path.exists(destination_file):

    url = media_url
    target_name = url_to_path(url)
    target_path = file_path(
        f'download/{target_name}', root=True)
    # target_path = task.file_path(
        # f'download/{target_name}', root=True)
    temp_path = file_path('download.tmp')
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
        # if os.path.isfile(target_path) and not opts.refresh:
        if os.path.isfile(target_path):
            logger.info(
                f'File exists, skipping download of {media_url} to {target_path} ({pretty_bytes(total_size)})')
            return DownloadResult(file_path=target_path, source_url=url)

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
                percent = download_size / total_size
                task.report_progress(percent)

        os.rename(temp_path, target_path)
        return DownloadResult(file_path=target_path, source_url=url)

def file_path(self, filename, scope=True) -> str:
    # if not scope:
    #     path = os.path.join(self.cache_path, 'shared', filename)
    # else:
    #     path = os.path.join(self.cache_path, 'job', self.id, filename)
    path = os.path.join(self.cache_path, filename)
    ensure_dir(path)
    return path

def ensure_dir(path: str):
    path = Path(path)
    os.makedirs(path.parent, exist_ok=True)
