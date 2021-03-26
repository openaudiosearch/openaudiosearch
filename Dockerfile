# build frontend
FROM node:14-alpine as frontend-build
WORKDIR /build
COPY /frontend /build
RUN yarn && yarn run build

# prepare main image
FROM python:3.9.0-slim as base

# install basics
RUN apt-get update && apt-get install -q -y portaudio19-dev gcc nodejs npm curl ffmpeg sox
# install poetry
ENV POETRY_HOME=/opt/poetry \
  POETRY_VIRTUALENVS_CREATE=false \
  PYTHONPATH=/app \
  PATH="/opt/poetry/bin:${PATH}"
RUN curl -fsS -o get-poetry.py https://raw.githubusercontent.com/python-poetry/poetry/master/get-poetry.py && \
  python get-poetry.py -y && . $POETRY_HOME/env

# install python deps
FROM base
WORKDIR /app/oas_core
# copy poetry package files
COPY oas_core/poetry.lock oas_core/pyproject.toml ./
# install dependencies
RUN . $POETRY_HOME/env && \
  poetry install --no-dev --no-interaction --no-root

# copy code
COPY . /app
COPY --from=frontend-build /build/dist /app/frontend/dist

# setup runtime
WORKDIR /app/oas_core
ENV STORAGE_PATH="/data"

# run with poetry
CMD ["poetry", "run", "python", "server.py"]
