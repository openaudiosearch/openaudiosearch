<h1 align="center">Open Audio Search</h1>
<div align="center">
 <strong>
    A full text search engine with automatic speech recognition for podcasts
  </strong>
</div>

<br />

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Docs: Rust](https://img.shields.io/badge/Docs-Rust-blue.svg)](https://openaudiosearch.github.io/openaudiosearch/rustdocs/oas_core)
[![Docs: Book](https://img.shields.io/badge/Docs-Book-blue.svg)](https://openaudiosearch.github.io/openaudiosearch/book/)


## What is it?

**Open Audio Search** uses automatic speech recognition to extract text from audio, which is then indexed in a search engine to enable recommendation of similiar broadcasts to users.
With **Open Audio Search**, we want to make the archives of community media, radio stations, podcasts searchable and discoverable, through open source tech and APIs.


## Main Features

* *Core backend* written in [Rust](https://rust-lang.org), providing a REST API and managing the indexing pipeline
* *Document database* using [CouchDB](https://couchdb.apache.org/)
* *Full-text search* using [Elasticsearch Community Edition](https://www.elastic.co/downloads/elasticsearch-oss)
* *Web user interface* using [React](https://reactjs.org/)
* *Task queue* with tasks written in [Python](https://python.org) (using [Celery](https://docs.celeryproject.org/) and [Redis](https://redis.io/))
* *Automatic Speech Recognition* using [Vosk toolkit](https://alphacephei.com/vosk/) ([Kaldi](http://kaldi-asr.org/) under the hood)


## Install & run with Docker

This project includes a Dockerfile to build a docker image for the backend and worker. It also includes a `docker-compose.yml` file to easily launch OAS together with Elastic Search and Redis.

To get started, install [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/). You'll need a quite recent version of both.

Then, run the following commands:
```sh
git clone https://github.com/openaudiosearch/openaudiosearch
cd open-audio-search
docker-compose build
docker-compose up
```

It takes a little while for Elastic to start up. Then, the OAS user interface and API are available at [`http://localhost:8080`](http://localhost:8080).

For the speech recognition to work, you'll need to download the models. Run this command once, it will download the models into the `./data/oas` volume:
```sh
docker-compose exec worker python download_models.py
```

Elastic Search wants quite a lot of free disc space. If the threshold is not met, it refuses to do anything. Run the script at `oas_worker/scripts/elastic-disable-threshold.sh` to disable the disc threshold (does not persist across Elastic restarts):
``` sh
docker-compose exec worker bash scripts/elastic-disable-threshold.sh
```

## Run locally for developing

To run OAS locally for developing or testing you should install the following requirements beforehand:
- For the core: [Rust](https://rust-lang.org), which is most easily installed with [Rustup](https://rustup.rs/).
- For the worker: [Python 3](https://python.org) and [poetry](https://python-poetry.org/docs/). Also requires [ffmpeg](https://www.ffmpeg.org/).
- For the frontend: [Node.js](https://nodejs.org/en/) and npm or yarn

*Clone this repository*
```sh
git clone https://github.com/openaudiosearch/openaudiosearch
```

*Start CouchDB, Elastisearch and Redis via Docker*
```sh
docker-compose -f docker-compose.dev.yml up
```

*Build an run the core*
```sh
cargo run -- run
```

*Run the frontend in development mode* 
```sh
cd frontend
yarn
yarn start
```

*Run the worker*
```sh
cd oas_worker
poetry install
./start-worker.sh
```

The live-reloading UI development server serves the UI at [http://localhost:4000](http://localhost:4000).
The OAS API is served at [http://localhost:8080](http://localhost:8080/). 
REST API docs are automatically generated and served at [http://localhost:8080/swagger-ui](http://localhost:8080/swagger-ui)

### Development tips and tricks

Have a look at the [development guide](./docs/development.md).

## Configuration

OAS is configured through environment variables or command line arguments. The following table lists all environment variables. Some apply to both the core and the worker, and some only to either.

|variable|default|applies to|description|
|-|-|-|-|
|`STORAGE_PATH`|`./data/oas`|worker|Storage path for models, cached files and other assets|
|`REDIS_URL`|`redis://localhost:6379/0`|both|URL to Redis server|
|`ELASTICSEARCH_URL`|`http://localhost:9200/`|core|URL to Elasticsearch server (trailing slash is required)|
|`ELASTICSEARCH_PREFIX`|`oas`|core|Prefix for all Elasticsearch indexes created by OAS|
|`COUCHDB_URL`|`http://admin:password@localhost:5984/oas`|core|URL to CouchDB server and database|
|`HTTP_HOST`|`0.0.0.0`|core|Interface for the HTTP server to bind to|
|`HTTP_PORT`|`8080`|core|Port for HTTP server to listen on|


## License
[AGPL v3](LICENSE)


## Documentation
The official documentation is hosted on tbd.


## Discussion and Development
Most development discussions take place on github in this repo. Further, tbd.


## Contributing to Open Audio Search

All contributions, bug reports, bug fixes, documentation improvements, enhancements, and ideas are welcome.

Code of conduct, tbd.
