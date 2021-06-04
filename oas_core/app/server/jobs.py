import json
import base64
from celery import chain, states
from celery.result import AsyncResult

from app.core.celery import app
from app.tasks.tasks import download, prepare, asr, nlp, index
from app.core.redis import redis


class Jobs(object):
    def __init__(self):
        self.redis = redis
        self.app = app
        self.queue = "celery"

    def get_jobs(self):
        # TODO: This does not fetch the currently running jobs, only the completed
        # and enqueued jobs.
        jobs = []
        # get completed jobs
        for key in self.redis.scan_iter("celery-task-meta-*"):
            encoded = self.redis.get(key)
            decoded = json.loads(encoded)
            job = {
                "task": decoded.get("name"),
                "args": decoded.get("args"),
                "id": decoded.get("task_id"),
                "status": decoded.get("status"),
                "date_done": decoded.get("date_done"),
                "result": decoded.get("result")
            }
            jobs.append(job)

        # get enqueued jobs
        queued_jobs = self.redis.lrange(self.queue, 0, -1)
        for encoded in queued_jobs:
            decoded = json.loads(encoded)
            body_encoded = base64.b64decode(decoded["body"])
            body_decoded = json.loads(body_encoded)
            job = {
                "task": decoded["headers"]["task"],
                "id": decoded["headers"]["id"],
                "args": body_decoded[0],
                "status": "PENDING",
                "date_done": None,
                "result": None
            }
            jobs.append(job)

        return jobs

    def get_job(self, job_id):
        state = AsyncResult(job_id, app=self.app)
        result = None
        if state.ready():
            result = state.get()
        return {
            'id': state.id,
            'task': state.name,
            'status': state.status,
            'date_done': state.date_done,
            'result': result,
            'args': state.args
        }

    def create_transcript_job(self, media_url):
        nlp_opts = {'pipeline': 'ner'}
        result = chain(
            download.s(media_url),
            prepare.s(16000),
            asr.s('vosk'),
            nlp.s(nlp_opts),
            index.s()
            )()
        return result
