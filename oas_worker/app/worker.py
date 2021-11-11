import argparse
from pprint import pprint
import httpx
import time
import os
from urllib.parse import urlparse, urlunparse
from mpire import WorkerPool
from mpire.exception import StopWorker
#  from mpire.dashboard import start_dashboard, connect_to_dashboard

from app.config import config
from app.logging import log
from app.client import JobClient

DASHBOARD_PORT = 8765

class Worker(object):
    def __init__(self, config=config):
        self.config = config
        self.jobs: Dict[str, Any] = {}
        self.pool = None
        self.client = JobClient(self.config)

    def job(self, name=None, default_concurrency=1):
        """
        Register a job
        """
        def decorator(fn):
            if name in self.jobs:
                raise Exception(f"Job {name} already registered, cannot register a job with the same name twice.")
            job_fn = JobFn(name, fn, concurrency=default_concurrency)
            self.jobs[name] = job_fn
            return job_fn

        return decorator


    def start(self):
        if not self.jobs:
            raise RuntimeError("No jobs registered.")
        jobs = []
        for (name, job) in self.jobs.items():
            for i in range(job.concurrency):
                jobs.append((job.name, i))
        n_jobs = len(jobs)
        with WorkerPool(n_jobs=n_jobs, shared_objects=self) as pool:
            self.pool = pool
            pool.map(work_loop, jobs)


    def single(self, typ):
        log.info(f"fetching single job for `{typ}`")
        work_fn(self, typ)


    def stop(self):
        if self.pool is None:
            return
        self.pool.terminate()

    def workdir(self, job_id):
        workdir = self.config.local_dir(f"job/{job_id}")
        return workdir

    #  def start_dashboard(self):
    #      start_dashboard(range(DASHBOARD_PORT, DASHBOARD_PORT + 2))


def work_fn(worker, typ, i=0, ctx=None):
    job = worker.jobs[typ]
    config = worker.config
    if ctx is None:
        ctx = Context(worker, typ, i)

    try:
        # this blocks until a job is available.
        info = worker.client.poll_next_job(typ)

        job_id = info["id"]
        args = info["args"]
        #  opts = info["subjects"]

        ctx.set_job(info)
        ctx.log.info(f"start job `{typ}:{job_id}`")

        start = time.perf_counter()
        try:
            #  result = job.fn(ctx, args, opts)
            result = job.fn(ctx, args)
            duration = round(time.perf_counter() - start, 3)
            worker.client.set_completed(job_id, patches=result.get("patches"), meta=result.get("meta"), duration=duration)
            ctx.log.info(f"completed job `{typ}:{job_id}` in {duration}s")
        except BaseException as err:
            duration = round(time.perf_counter() - start, 3)
            error = str(err) or 'Unkown error'
            ctx.log.error(f"failed job `{typ}:{job_id}` after {duration}s: {error}", exception=err)
            ctx.log.exception(err)
            worker.client.set_failed(job_id, error=error, duration=duration)

    except httpx.RequestError as err:
        ctx.log.error(f'Failed to fetch {err.request.url!r}: {err}')
    except httpx.HTTPStatusError as err:
        ctx.log.error(f"Failed to fetch {err.request.url!r}: {err.response.status_code} {err.response.reason_phrase}")
    except BaseException as err:
        ctx.log.error(f"Failed while processing a job: {err}")


def work_loop(worker, typ, i):
    pid = os.getpid()
    ctx = Context(worker, typ, i)
    ctx.log.info(f"start worker thread (pid {pid})")
    while True:
        try:
            work_fn(worker, typ, ctx=ctx)
            time.sleep(1)
        except StopWorker:
            break
        except BaseException as err:
            ctx.log.error(f"worker failed: {err}", exception=err)
            time.sleep(1)

    ctx.log.info(f"stop worker thread (pid {pid})")


def url_without_password(parsed):
    replaced = parsed._replace(netloc="{}:{}".format(parsed.hostname, parsed.port))
    return replaced.geturl() 


class Context(object):
    def __init__(self, worker, name, i):
        self.worker = worker
        self.name = name
        self.worker_id = i
        self.log = log.bind(name=name,id=i)
        self.job = None
        self.job_id = None

    def set_progress(self, progress, meta=None):
        self.log.debug(f"  Progress: {progress}")
        return self.worker.client.set_progress(self.job_id, progress, meta=meta)

    def get(self, url):
        return self.worker.client.get(url)

    def set_job(self, job):
        self.job_id = job["id"]
        self.job = job

    def workdir(self):
        if self.job_id is None:
            raise RuntimeException("No job set")
        return self.worker.workdir(self.job_id)


class JobFn(object):
    def __init__(self, name, fn, concurrency=1):
        self.name = name
        self.concurrency = concurrency
        self.fn = fn

    def __call__(self, ctx, args, opts):
        if not opts:
            opts = None
        return self.fn(ctx, args, opts)


worker = Worker()
