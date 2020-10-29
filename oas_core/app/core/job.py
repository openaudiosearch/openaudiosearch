import os
from typing import Any, Dict
from pathlib import Path
import logging
import traceback
import sys
import tempfile
from functools import wraps
from queue import Queue
from enum import Enum

from .util import uuid

class State(Enum):
    queued = 'queued'
    running = 'running'
    blocked = 'blocked'
    completed = 'completed'
    error = 'error'
    
class Worker(object):

    def __init__(self, cache_path=None):
        self.cache_path = cache_path or tempfile.mkdtemp()
        self.registered_tasks: Dict[str, Any] = {}
        self.queue: Queue = Queue()

    def task(self, name=None,result=None):
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
        
    def enqueue_job(self, task_name: str, args, opts):
        if not task_name in self.registered_tasks:
            raise NameError(f'Task not found: {task_name}')

        id = uuid()
        cache_path = os.path.join(self.cache_path, id)

        job = Job(id, cache_path=cache_path)

        task_fn = self.registered_tasks[task_name]
        job.add_task(task_fn, args, opts)
        print(f'enqueue job {job}')

        self.queue.put(job)


    def run(self):
        while True:
            print(f'QUEUE LENGTH: {len(self.queue.queue)}')
            job = self.queue.get()
            print(f'START {job.id}')
            job.run()
            print(f'FIN {job.id}')
            self.queue.task_done()

class Job(object):

    def __init__(self, id, cache_path=None, args=None, opts=None):
        self.id = id
        self.cache_path = cache_path
        self.state: State = State.queued
        self.queue: Queue = Queue()
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
        return self.fn(task, args, opts)