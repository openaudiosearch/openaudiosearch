#!/bin/sh
poetry run celery -A app.celery worker --loglevel=DEBUG
