#!/bin/sh
celery -A app.tasks.tasks worker --loglevel=INFO --concurrency=${CONCURRENCY:=1}
