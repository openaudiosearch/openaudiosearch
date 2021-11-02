import argparse
import logging
import httpx
import time
import os
from urllib.parse import urlparse, urlunparse
from mpire import WorkerPool

DEFAULT_OAS_URL = 'http://admin:password@localhost:8080/api/v1'

class Config(object):
    def __init__(self):
        self.base_url = os.environ.get("OAS_URL") or DEFAULT_OAS_URL
        self.log_exceptions = (os.environ.get("LOG_ERROR", False) != False)
        try:
            self.base_url_parsed = urlparse(self.base_url, 'http')
        except BaseException as err:
            logging.error(f"Failed to parse OAS_URL: {err}")


class JobClient(object):
    def __init__(self, config):
        self.base_url = config.base_url

    def next_job(self, typ):
        url = f"{self.base_url}/work/{typ}"
        while True:
            res = httpx.post(url)
            if res.status_code == 200:
                res = res.json()
                return res
            elif res.status_code == 204:
                logging.debug("No work to do, waiting and polling")
                time.sleep(5)
            else:
                raise res.raise_for_status()

    def set_completed(self, job_id, patches=None, meta=None, duration=None):
        body = {
            "patches": patches,
            "meta": meta,
            "duration": duration
        }
        url = f"{self.base_url}/job/{job_id}/completed"
        res = httpx.put(url, json=body)
        res = res.json()
        return res

    def set_progress(self, job_id, progress, meta=None):
        body = {
            "progress": progress,
            "meta": meta
        }
        url = f"{self.base_url}/job/{job_id}/progress"
        res = httpx.put(url, json=body)
        res = res.json()
        return res

    def set_failed(self, job_id, error=None, meta=None, duration=None):
        body = {
            "error": error,
            "meta": meta,
            "duration": duration
        }
        url = f"{self.base_url}/job/{job_id}/failed"
        res = httpx.put(url, json=body)
        res = res.json()
        return res

    def get(self, url):
        url = f"{self.base_url}{url}"
        res = httpx.get(url)
        res = res.json()
        return res


class Worker(object):
    def __init__(self, config=None):
        self.config = config or Config()
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
        tasks = []
        for (name, task) in self.tasks.items():
            for i in range(task.concurrency):
                tasks.append((task.name, i))
        n_jobs = len(tasks)
        with WorkerPool(n_jobs=n_jobs, shared_objects=self) as pool:
            self.pool = pool
            results = []
            pool.map(work_loop, tasks)


    def single(self, typ):
        work_fn(self, typ)


    def stop(self):
        if self.pool is None:
            return
        self.pool.terminate()


def work_fn(worker, typ, i=0, ctx=None):
    task = worker.tasks[typ]
    config = worker.config
    if ctx is None:
        ctx = Context(worker, typ, i)

    try:
        job = worker.client.next_job(typ)

        job_id = job["id"]
        args = job["args"]
        opts = job["subjects"]

        ctx.set_job(job)
        ctx.log.info(f"start job `{typ}` (id {job_id})")

        start = time.perf_counter()
        try:
            result = task.fn(ctx, args, opts)
            duration = round(time.perf_counter() - start, 3)
            worker.client.set_completed(job_id, patches=result.get("patches"), meta=result.get("meta"), duration=duration)
            ctx.log.info(f"Job {job_id} completed")
        except BaseException as err:
            print(err)
            error = str(err)
            ctx.log.error(f"Job {typ} (id {job_id}) failed: {error}")
            if config.log_exceptions: ctx.log.exception(err)
            worker.client.set_failed(job_id, error=error)

    except httpx.ConnectError as err:
        urlstring = url_without_password(worker.config.base_url_parsed)
        ctx.log.error(f"Failed to connect to {urlstring}: {err}")
        if config.log_exceptions: ctx.log.exception(err)
    except KeyboardInterrupt as err:
        raise err
    except BaseException as err:
        logging.error(err)
        if config.log_exceptions: logging.exception(err)


def work_loop(worker, typ, i):
    pid = os.getpid()
    ctx = Context(worker, typ, i)
    ctx.log.info(f"start (pid {pid})")
    while True:
        work_fn(worker, typ, ctx=ctx)
        time.sleep(1)

    ctx.log.info(f"end (pid {pid})")


def url_without_password(parsed):
    replaced = parsed._replace(netloc="{}:{}".format(parsed.hostname, parsed.port))
    return replaced.geturl() 


class Context(object):
    def __init__(self, worker, name, i):
        self.worker = worker
        self.name = name
        self.worker_id = i
        self.log = logging.getLogger(f'{name}:{i}')
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

@worker.task("asr", default_concurrency=1)
def asr(ctx, args, opts):
    meta = {
        "model": "foo1"
    }
    ctx.set_progress(0.0, meta=meta)

    time.sleep(2)
    ctx.set_progress(20)
    time.sleep(2)
    ctx.set_progress(40)
    time.sleep(2)
    ctx.set_progress(60)
    
    media_id = args["media_id"]
    guid = "oas.Media_" + media_id
    media = ctx.get("/media/" + media_id)
    transcript = {
        "text": "foo bar baz " + media_id + " " + str(media["duration"]),
        "parts": [
            { "start": 0.2, "end": 1.0, "conf": 1.0, "word": "foo" },
            { "start": 0.2, "end": 1.0, "conf": 1.0, "word": "bar" },
            { "start": 0.2, "end": 1.0, "conf": 1.0, "word": "baz" },
        ]
    }
    patch = [
        { "op": "replace", "path": "/transcript", "value": transcript },
    ]
    patches = {
        guid: patch
    }
    return {
        "meta": {
            "asrfoo": "bar"
        },
        "patches": patches
    }

@worker.task("nlp", default_concurrency=2)
def nlp(ctx, args, opts):
    ctx.log.info("hello info nlp")
    #  print("nlp", ctx.worker_id, "job", ctx.job_id, "ARGS", args, opts)
    time.sleep(1)
    ctx.log.info("done")
    return {
        "meta": {
            "nlpfoo": "bazoo"
        }
    }

if __name__ == '__main__':
    loglevel = os.environ.get('LOGLEVEL', 'INFO').upper()
    logging.basicConfig(
        format=f'[%(asctime)s %(levelname)s %(name)s] %(message)s', level = loglevel)

    logging.info(
        "Start OAS worker, PID {}".format(os.getpid()))

    parser = argparse.ArgumentParser(description='OAS Worker')
    parser.add_argument('--single', type=str, help='Run a single job of this typ')
    args = parser.parse_args()

    try:
        if args.single is not None:
            logging.info(f"fetching single job for `{args.single}`")
            worker.single(args.single)
            logging.info(f"done")
            exit(0)
        else:
            worker.start()
    except KeyboardInterrupt:
        logging.warning("Received Ctrl-C, shutting down...")
        worker.stop()
        logging.warning("Stopped all workers")
        exit(0)


