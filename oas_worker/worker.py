import argparse
import logging
import httpx
import time
from os import getpid
from mpire import WorkerPool

OAS_URL = 'http://admin:password@localhost:8080/api/v1'


class Worker(object):
    def __init__(self):
        self.tasks: Dict[str, Any] = {}

    def task(self, name=None, default_concurrency=1):
        """
        Register a task
        """
        def decorator(fn):
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
            results = []
            pool.map(work_fn, tasks)


def work_fn(worker, name, i):
    pid = getpid()
    task = worker.tasks[name]
    while True:
        try:
            job = get_job(name)
            ctx = Context(name, i, job)
            ctx.log.info(f"start (pid {pid})")
            job_id = job["id"]
            args = job["args"]
            opts = job["subjects"]
            result = task.fn(ctx, args, opts)
            post_result(job_id, result)
        except BaseException as err:
            ctx.log.error(err)
            time.sleep(1)

    args = {}
    opts = None
    task.fn(ctx, args, opts)
    ctx.log.info(f"end (pid {pid})")


def get_job(typ):
    url = f"{OAS_URL}/work/{typ}"
    res = httpx.post(url)
    res = res.json()
    return res

def post_result(job_id, result):
    url = f"{OAS_URL}/job/{job_id}"
    res = httpx.patch(url, json=result)
    res = res.json()
    return res


class Context(object):
    def __init__(self, name, i, job):
        self.name = name
        self.worker_id = i
        self.job_id = job["id"]
        self.job = job
        self.log = logging.getLogger(f'{name}:{i}')

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
    ctx.log.info("hello info asr")
    #  print("asr", ctx.worker_id, "job", ctx.job_id, "ARGS", args, opts)
    time.sleep(2)
    ctx.log.info("done")
    return {
        "meta": {
            "asrfoo": "bar"
        }
    }

@worker.task("nlp", default_concurrency=1)
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
    logging.basicConfig(
        format=f'[%(asctime)s %(levelname)s %(name)s] %(message)s', level = logging.DEBUG)

    logging.info(
        "Start OAS worker, PID {}".format(getpid()))

#      parser = argparse.ArgumentParser(description='OAS Worker')
#      #  parser.add_argument('post_id', metavar='P', type=str, help='Post ID to process')
#      args = parser.parse_args()
    worker.start()

