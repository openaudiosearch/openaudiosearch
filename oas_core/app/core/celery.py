from celery import Celery

app = Celery('tasks', broker='redis://localhost', backend='redis://localhost', result_extended=True)
