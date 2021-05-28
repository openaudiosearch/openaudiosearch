from app.core.celery import app
from app.tasks.tasks import download, prepare, asr, nlp, index
from celery import chain
from celery.result import AsyncResult

class Jobs(object):
    def __init__(self):
        self.app = app

    def get_jobs(self):
        active_jobs = self.app.control.inspect().active()
        scheduled_jobs = self.app.control.inspect().scheduled()
        jobs = list(active_jobs.values()) + list(scheduled_jobs.values())
        flattened_jobs = [y for x in jobs for y in x]
        return flattened_jobs

    def get_job(self, job_id):
        res = AsyncResult(job_id, app=self.app)
        return {'status': res.status}

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
