# Open Audio Search

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)


## What is it?

**Open Audio Search** uses automatic speech recognition to extract text from audio, which is then indexed in a search engine to enable recommendation of similiar broadcasts to users.  
With **Open Audio Search**, we want to make the archives of community media, radio stations, podcasts searchable and discoverable, through open source tech and APIs.


## Main Features

* *Automatic Speech Recognition* using [Vosk toolkit](https://alphacephei.com/vosk/) ([Kaldi](http://kaldi-asr.org/) under the hood)
* *Task queue app engine* using [Redis](https://redis.io/) for caching.
* *[Elasticsearch Community Edition](https://www.elastic.co/downloads/elasticsearch-oss)* baked indexing and search.
* *React Single Page Application* as User Interface.

## Install & run with Docker

This project includes a Dockerfile to build a docker image for the backend and worker. It also includes a `docker-compose.yml` file to easily launch OAS together with Elastic Search and Redis.

To get started, install [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/). You'll need a quite recent version of both.

Then, run the following commands:
```sh
git clone https://github.com/arso-project/open-audio-search
cd open-audio-search
docker-compose build
docker-compose up
```

The OAS user interface and API are now available at `http://localhost:8080`.

For the speech recognition to work, you'll need to download the models. Run this command once, it will download the models into the `./data/oas` volume:
```sh
docker-compose exec worker python task-run.py download_models
```

## Run locally for developing

To develop locally you may want to run OAS without Docker. You should install the following requirements beforehand:
- for the frontend: [Node.js](https://nodejs.org/en/) and npm or yarn
- for the backend: Python and [poetry](https://python-poetry.org/docs/)
- at runtime: [ffmpeg](https://www.ffmpeg.org/)

*Clone this repository*
```sh
git clone https://github.com/arso-project/open-audio-search
```

*Prepare and build frontend* 
```sh
cd frontend
yarn
yarn build
```

*Prepare and install backend*
```sh
cd oas_core
poetry install
```

*Start elasticsearch and redis via Docker*
```sh
docker-compose -f docker-compose.dev.yml up
```

*Start OAS server and worker*
```sh
cd oas_core
poetry run python server.py
# in another terminal:
poetry run python worker.py
```

Open demo in browser at [http://localhost:8080](http://localhost:8080/).


#### Configuration

OAS is configured through an `.env` file in the directory from where you invoke it. To customize the configuration, copy [`.env.default`](`oas_core/.env.default`) in the `oas_core` folder to `.env` and adjust the values.

By default, all data is stored in `./data/oas`.

## Development Setup

Have a look at the [development guide](./docs/development.md).


## License
[AGPL v3](LICENSE)


## Documentation
The official documentation is hosted on tbd.


## Discussion and Development
Most development discussions take place on github in this repo. Further, tbd.


## Contributing to Open Audio Search

All contributions, bug reports, bug fixes, documentation improvements, enhancements, and ideas are welcome.

Code of conduct, tbd.
