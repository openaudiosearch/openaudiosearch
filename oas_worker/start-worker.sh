#!/bin/sh
poetry run celery -A app.tasks.tasks worker --loglevel=DEBUG --concurrency=$CONCURRENCY
