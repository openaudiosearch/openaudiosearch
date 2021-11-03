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
        self.tasks: Dict[str, Any] = {}
        self.pool = None
        self.client = JobClient(self.config)

    def task(self, name=None, default_concurrency=1):
        """
        Register a task
        """
        def decorator(fn):
            if name in self.tasks:
                raise Exception(f"Task {name} already registered, cannot register a task with the same name twice.")
            task_fn = TaskFn(name, fn, concurrency=default_concurrency)
            self.tasks[name] = task_fn
            return task_fn

        return decorator


    def start(self):
        if not self.tasks:
            raise RuntimeError("No jobs registered.")
        tasks = []
        for (name, task) in self.tasks.items():
            for i in range(task.concurrency):
                tasks.append((task.name, i))
        n_jobs = len(tasks)
        with WorkerPool(n_jobs=n_jobs, shared_objects=self) as pool:
            self.pool = pool
            pool.map(work_loop, tasks)


    def single(self, typ):
        log.info(f"fetching single job for `{typ}`")
        work_fn(self, typ)


    def stop(self):
        if self.pool is None:
            return
        self.pool.terminate()

    #  def start_dashboard(self):
    #      start_dashboard(range(DASHBOARD_PORT, DASHBOARD_PORT + 2))


def work_fn(worker, typ, i=0, ctx=None):
    task = worker.tasks[typ]
    config = worker.config
    if ctx is None:
        ctx = Context(worker, typ, i)

    try:
        # this blocks until a job is available.
        job = worker.client.poll_next_job(typ)

        job_id = job["id"]
        args = job["args"]
        opts = job["subjects"]

        ctx.set_job(job)
        ctx.log.info(f"start job `{typ}:{job_id}`")

        start = time.perf_counter()
        try:
            result = task.fn(ctx, args, opts)
            duration = round(time.perf_counter() - start, 3)
            worker.client.set_completed(job_id, patches=result.get("patches"), meta=result.get("meta"), duration=duration)
            ctx.log.info(f"completed job `{typ}:{job_id}` in {duration}s")
        except BaseException as err:
            duration = round(time.perf_counter() - start, 3)
            error = str(err) or 'Unkown error'
            ctx.log.error(f"failed job `{typ}:{job_id}` after {duration}s: {error}", exception=err)
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


class TaskFn(object):
    def __init__(self, name, fn, concurrency=1):
        self.name = name
        self.concurrency = concurrency
        self.fn = fn

    def __call__(self, ctx, args, opts):
        if not opts:
            opts = None
        return self.fn(ctx, args, opts)


worker = Worker()
