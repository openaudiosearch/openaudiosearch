<h1 align="center">Open Audio Search</h1>
<div align="center">
 <strong>
    A full text search engine with automatic speech recognition for podcasts
  </strong>
</div>

<br />

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Docs: Book](https://img.shields.io/badge/Docs-Book-blue.svg)](https://openaudiosearch.github.io/openaudiosearch/book/)
[![Docs: Rust](https://img.shields.io/badge/Docs-Rust-blue.svg)](https://openaudiosearch.github.io/openaudiosearch/rustdocs/oas_core)
[![Chat: Discord](https://img.shields.io/badge/Chat-Discord-green.svg)](http://discord.openaudiosearch.org)

## What is it?

**Open Audio Search** is a search engine for audio files. It indexes RSS feeds and uses automatic speech recognition to extract text from audio. The feeds and transcripts are indexed in a search engine, empowering users to use full-text search on the transcripts and listen to the audio files while jumping right to search result snippets.

With **Open Audio Search**, we want to make the archives of community media, radio stations and podcasts searchable and discoverable, through open source tech and APIs.

## Status

Open Audio Search is still **in development**. No API stability guarantees yet. Don't run this for anything serious or on public instances at the moment.

## Main Features

* *Core backend* written in [Rust](https://rust-lang.org), providing a REST API and managing the indexing pipeline
* *Document database* using [CouchDB](https://couchdb.apache.org/)
* *Full-text search* using [Elasticsearch Community Edition](https://www.elastic.co/downloads/elasticsearch-oss)
* *Web user interface* using [React](https://reactjs.org/)
* *Task queue* with tasks written in [Python](https://python.org) (using [Celery](https://docs.celeryproject.org/) and [Redis](https://redis.io/))
* *Automatic Speech Recognition* using [Vosk toolkit](https://alphacephei.com/vosk/) ([Kaldi](http://kaldi-asr.org/) under the hood)

## Docs, references and utilities


* For installation and usage see further down in here
* HTTP API docs are auto-generated (still incomplete) and can be viewed on the [demo site](https://demo.openaudiosearch.org/swagger-ui/). Some useful examples on how to use the HTTP API can be found in the [rest-examples](scripts/rest-examples/README.md) folder
* The worker has a [README](oas_worker/README.md) with docs on how to run and create jobs
* When working with the Rust core, the [Rust API docs](https://openaudiosearch.github.io/openaudiosearch/rustdocs/oas_core/) may help you
* We have some resources and docs in a [documentation book](https://openaudiosearch.github.io/openaudiosearch/book/) (still incomplete and partially outdated)
* A user guide focusing on the user perspective (incl. How-to-Guides and FAQs) can be found in the [user docs section](https://openaudiosearch.github.io/openaudiosearch/book/user.html) of our [documentation book](https://openaudiosearch.github.io/openaudiosearch/book/) (again, still incomplete and partially outdated)

## Installation and usage

### Installation with docker

Requirements: [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/). You'll need a quite recent version of both.


#### Use prebuilt nightly images

We provide `nightly` Docker images which are built and pushed to Docker Hub after each commit to the `main` branch. We will provide stable images once we reach our first beta release.

For a quick start, copy [`docker-compose.nightly.yml`](https://raw.githubusercontent.com/openaudiosearch/openaudiosearch/main/docker-compose.nightly.yml) to an empty directory and start it with Docker Compose. This will download the latest images for the backend and worker and start them together with CouchDB, Elasticsearch and Redis.

```
mkdir openaudiosearch && cd openaudiosearch
wget https://raw.githubusercontent.com/openaudiosearch/openaudiosearch/main/docker-compose.nightly.yml
mv docker-compose.nightly.yml docker-compose.yml
docker-compose up
```

For the speech recognition to work you will need to download the models first. Run this command once, it will download the models into the `./data/oas` volume:

```sh
docker-compose exec worker python download_models.py
```

#### Build Docker images from source

This project includes a Dockerfile to build docker images for the backend and worker. It also includes a `docker-compose.yml` file to easily launch OAS together with CouchDB, Elasticsearch and Redis.

Then, the following commands will build the Docker image from source and start it together with all required services.
```sh
git clone https://github.com/openaudiosearch/openaudiosearch
cd openaudiosearch
docker-compose build
docker-compose up
```

It takes a little while for Elasticsearch to start up. Then, the OAS user interface and API are available at [`http://localhost:8080`](http://localhost:8080).


Elasticsearch wants quite a lot of free disc space. If the threshold is not met, it refuses to do anything. Run the script at `oas_worker/scripts/elastic-disable-threshold.sh` to disable the disc threshold (does not persist across Elasticsearch restarts):
``` sh
docker-compose exec worker bash scripts/elastic-disable-threshold.sh
```

### Configuration

OAS is configured through environment variables or command line arguments. The following table lists all environment variables. Some apply to both the core and the worker, and some only to either.

#### `OAS_ADMIN_PASSWORD`

default: `password`
applies to: core

Default password for `admin` user.

#### `OAS_URL`

default: `http://admin:password@localhost:8080/api/v1`
applies to: worker

HTTP URL of the OAS core API, including a valid username and password.

#### `STORAGE_PATH`

default: `./data/oas`
applies to: worker

Storage path for models, cached files and other assets.


#### `REDIS_URL`

default: `redis://localhost:6379/0`
applies to: both

URL to Redis server


#### `ELASTICSEARCH_URL`

default: `http://localhost:9200/oas`
applies to: core

URL to Elasticsearch server and index prefix


#### `COUCHDB_URL`

default: `http://admin:password@localhost:5984/oas`
applies to: core

URL to CouchDB server and database prefix


#### `HTTP_HOST`

default: `0.0.0.0`
applies to: core

Interface for the HTTP server to bind to


#### `HTTP_PORT`

default: `8080`
applies to: core

Port for HTTP server to listen on


#### `FRONTEND_PROXY`

applies to: core

If set to a HTTP URL, all requests for the web UI are proxied to this address

#### `OAS_CONCURRENCY_ASR`

default: `1`
applies to: worker

Sets the number of worker processes/threads for ASR processing.


## Development and local setup

To run OAS locally for developing or testing you should install the following requirements beforehand:
- For the core: [Rust](https://rust-lang.org), which is most easily installed with [Rustup](https://rustup.rs/). The minimum supported version of Rust (MSRV) is **1.54.0**. You might also need a C compiler and OpenSSL development headers. On Debian based systems, run `apt install gcc libssl-dev pkg-config`.
- For the worker: [Python 3](https://python.org) and [poetry](https://python-poetry.org/docs/). Also requires [ffmpeg](https://www.ffmpeg.org/).
- For the frontend: [Node.js](https://nodejs.org/en/) and npm or yarn.

*Clone this repository*
```sh
git clone https://github.com/openaudiosearch/openaudiosearch
```

*Start CouchDB, Elastisearch and Redis via Docker*
```sh
docker-compose -f docker-compose.dev.yml up
```

Optionally set a data directory for all services (to start from a clean slate):
```sh
OAS_DATA_DIR=/tmp/oas docker-compose -f docker-compose.dev.yml up
```

*Run the frontend in development mode* 
```sh
cd frontend
yarn
yarn start
```

*Build an run the core*

Compile and run the Rust core, while setting an environment variable to proxy the web UI from a local development server (see below):
```sh
cargo run -- --dev run
```
The `--dev` argument enables debug logging (`RUST_LOG=oas=debug`) and sets `FRONTEND_PROXY=http://localhost:4000`

To build and test in release mode, use
```sh
cargo run --release -- run
```

*Run the worker*
```sh
cd oas_worker
poetry install
./start-worker.sh
```

Now open [http://localhost:8080](http://localhost:8080) in a web browser. The UI is proxied to the live-reloading UI development server which runs at [http://localhost:4000](http://localhost:4000). The OAS API is served at `http://localhost:8080/api/v1`. REST API docs are automatically generated and served at [http://localhost:8080/swagger-ui](http://localhost:8080/swagger-ui).

### Development tips and tricks

Have a look at the [development guide](./docs/development.md).

## License

Open Audio Search is licensed under the [AGPL v3](LICENSE).

## Documentation

Documentation is still sparse. Docs are located in the [docs](docs) folder and rendered to HTML on the [documentation site](https://openaudiosearch.github.io/openaudiosearch/book/). We also host [API docs for the Rust core](https://openaudiosearch.github.io/openaudiosearch/rustdocs/oas_core). The REST API is documented from within Open Audio Search (TODO: Host and link to REST API docs).

## Contributing

All contributions, bug reports, bug fixes, documentation improvements, enhancements, and ideas are welcome. Please open issues or [talk to us on our Discord server](http://discord.openaudiosearch.org). We want to welcome anyone and commit to creating an inclusive environment.

Development discussions currently take place here on GitHub or on our Discord chat.

## Links & Thanks

Open Audio Search is a project by [arso collective](https://arso.xyz) and [cba](https://cba.fro.at) and supported by [Prototype Fund](https://prototypefund.de/project/open-audio-search/) and [netidee](https://www.netidee.at/open-audiosearch).

