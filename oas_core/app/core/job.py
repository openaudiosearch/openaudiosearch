import os
import json
import typing as T
from pathlib import Path
import logging
import traceback
import sys
import tempfile
from queue import Queue
from enum import Enum

from pydantic import BaseModel
from redis import StrictRedis as Redis, ConnectionPool
from typing import Any, Dict
# from functools import wraps

from app.core.util import uuid, get_typed_signature


class State(Enum):
    queued = 'queued'
    running = 'running'
    blocked = 'blocked'
    completed = 'completed'
    error = 'error'


JOBS = 'JOBS'
RESULTS = 'RESULTS'
TASKS = 'TASKS'


class TaskSpec(BaseModel):
    job_id: str
    task_name: str
    args: T.Any
    opts: T.Any


class Client(object):
    def __init__(self, redis_pool=None, redis_host='localhost', redis_port=6379, redis_db=0):
        self.pool = redis_pool or ConnectionPool(
            port=redis_port, host=redis_host, db=redis_db)
        self.redis = Redis(connection_pool=redis_pool)
        self.prefix = 'oas.queue.'
        self.key_results = self.key(RESULTS)
        self.key_tasks = self.key(TASKS)
        self.key_jobs = self.key(JOBS)
        self.read_timeout = 1

    def encode(self, obj):
        if isinstance(obj, BaseModel):
            return obj.json()
        return json.dumps(obj)

    def key(self, key: str):
        return self.prefix + key

    def queue_job(self, task_name: str, args, opts):
        id = uuid()
        self.queue_task(id, task_name, args, opts)
        return id
        # encoded = json.dumps(
        #     {'id': id, 'task': task_name, 'args': args, 'opts': opts})
        # self.redis.lpush(JOBS, encoded)

    def set_result(self, id, result):
        encoded = self.encode(result)
        print(f'SET RESULT {self.key_results} {id} : {encoded}')
        self.redis.hset(self.key_results, id, encoded)

    def get_result(self, id):
        encoded = self.redis.hget(self.key_results, id)
        if not encoded:
            return None
        # print(f'GET RESULT {self.key_results} {id} : {encoded}')
        decoded = json.loads(encoded)
        return decoded

    def queue_task(self, job_id: str, task_name: str, args, opts):
        spec = TaskSpec(job_id=job_id, task_name=task_name,
                        args=args, opts=opts)
        encoded = self.encode(spec)
        # args = self.encode(args)
        # opts = self.encode(opts)
        # encoded = json.dumps(
        #     {'id': job_id, 'task': task_name, 'args': args, 'opts': opts})
        self.redis.lpush(self.key_tasks, encoded)

    def dequeue_task(self):
        # blocking!
        try:
            encoded = self.redis.brpop(
                self.key_tasks,
                timeout=self.read_timeout)[1]
            if not encoded:
                return None
            decoded = json.loads(encoded)
            return TaskSpec(**decoded)
        except (ConnectionError, TypeError, IndexError):
            # Unfortunately, there is no way to differentiate a socket
            # timing out and a host being unreachable.
            return None


class Worker(object):

    def __init__(self, cache_path=None, client=None):
        self.client = client or Client()
        self.cache_path = cache_path or tempfile.mkdtemp()
        self.registered_tasks: Dict[str, Any] = {}
        self.queue: Queue = Queue()

    def task(self, name=None, result=None):
        print(f'register task: {name}')
        """
        Register a task
        """
        def decorator(fn):
            # print(f'register decorator: {name} {fn}')
            # task = Task(self, name, fn)
            wrapped_fn = WrappedTaskFn(name, fn)
            self.registered_tasks[name] = wrapped_fn

            # @wraps(fn)
            # def wrapper(*args, **kwargs):
            #     # print(f'register call!! {name}')
            #     raise NotImplementedError('May not call tasks directly')

            # wrapper.__task = task

            # print(f'worker self {self.registered_tasks}')
            # return wrapper
            return wrapped_fn

        return decorator

    def queue_job(self, task_name: str, args, opts, id=None):
        if not task_name in self.registered_tasks:
            raise NameError(f'Task not found: {task_name}')

        if not id:
            id = uuid()

        cache_path = os.path.join(self.cache_path, id)

        job = Job(id, cache_path=cache_path, client=self.client)

        task_fn = self.registered_tasks[task_name]
        job.add_task(task_fn, args, opts)

        self.queue.put(job)

    def run(self):
        while not self.queue.empty():
            print(f'QUEUE LENGTH: {len(self.queue.queue)}')
            job = self.queue.get()
            print(f'START {job.id}')
            job.run()
            print(f'FIN {job.id}')
            self.queue.task_done()


class Job(object):

    def __init__(self, id, client=None, cache_path=None, args=None, opts=None):
        self.id = id
        self.cache_path = cache_path
        self.state: State = State.queued
        self.queue: Queue = Queue()
        self.client = client
        # print(f'create job {id}: {self} {self.id}')
        # self.args = args
        # self.opts = opts

    def add_task(self, task_fn, args=None, opts=None):
        # accept both a task object and a wrapped task function
        # if task.__task:
        #     task = task.__task
        # print(task_fn)
        # task = task_fn.__task
        task_name = task_fn.name
        task = Task(self, task_name, task_fn)
        if args:
            task.set_args(args)
        if opts:
            task.set_opts(opts)
        self.queue.put(task)
        return task

    def log(self, message):
        logging.info(f'JOB LOG ({self.id}): {message}')

    def set_state(self, state: State):
        self.state = state

    def run(self):
        # input = self.args
        input = None
        self.set_state(State.running)
        while not self.queue.empty():
            task = self.queue.get()

            # TODO: multiprocessing
            task.run(input)

            if task.success():
                self.log(f'result {task.result}')
                # print(f'RES: {task.result}')
                input = task.result
                self.queue.task_done()
            else:
                self.log(f'error {task.error}')
                logging.exception(task.error)
                # traceback.print_tb(task.error.__traceback__)
                self.set_state(State.error)
                return
        self.client.set_result(self.id, input)

    def file_path(self, filename, scope=True) -> str:
        # if not scope:
        #     path = os.path.join(self.cache_path, 'shared', filename)
        # else:
        #     path = os.path.join(self.cache_path, 'job', self.id, filename)
        path = os.path.join(self.cache_path, filename)
        path = Path(path)
        os.makedirs(path.parent, exist_ok=True)
        return str(path)


class Task(object):
    def __init__(self, job, name, fn):
        self.job = job
        self.name = name
        self.fn = fn
        self.started = False
        self.opts = None
        self.args = None

        self.state = State.queued
        self.result = None
        self.error = None

    def set_opts(self, opts):
        self.opts = opts

    def set_args(self, args):
        self.args = args

    def set_state(self, state: State):
        self.log(f'state change {self.state} -> {state}')
        self.state = state

    def success(self) -> bool:
        return self.state == State.completed

    def log(self, message):
        self.job.log(f'[{self.name}] {message}')

    def report_progress(self, percent, message=None):
        self.log(f'PROGRESS {percent}')

    def run(self, args=None):
        if self.started:
            raise RecursionError('Task is already running')

        if args:
            self.set_args(args)

        self.set_state(State.running)
        self.started = True
        try:
            print(f'  RUN TASK {self.name} {self.args} {self.opts}')
            self.result = self.fn(self, self.args, self.opts)
            self.set_state(State.completed)
        except Exception as error:
            self.error = error
            self.set_state(State.error)
            traceback.print_exc()

    def file_path(self, filename):
        return self.job.file_path(os.path.join(self.name, filename))


class WrappedTaskFn(object):
    def __init__(self, name, fn):
        self.name = name
        self.fn = fn

    def __call__(self, task: Task, args, opts):
        if not opts:
            opts = None

        # upcast the parameter to the typed model classes from pydantic, if provided
        (args_model, opts_model) = self.get_typed_signature()
        if args_model and issubclass(args_model, BaseModel) and not isinstance(args, args_model):
            args = args_model(**args)
        if opts_model and issubclass(opts_model, BaseModel) and not isinstance(opts, opts_model):
            opts = opts_model(**opts)
        return self.fn(task, args, opts)

    def get_typed_signature(self):
        signature = get_typed_signature(self.fn)
        args = signature.parameters['args'].annotation
        opts = signature.parameters['opts'].annotation
        return (args, opts)
