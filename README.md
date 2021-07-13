# Open Audio Search

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)


## What is it?

**Open Audio Search** uses automatic speech recognition to extract text from audio, which is then indexed in a search engine to enable recommendation of similiar broadcasts to users.  
With **Open Audio Search**, we want to make the archives of community media, radio stations, podcasts searchable and discoverable, through open source tech and APIs.


## Main Features

* *Automatic Speech Recognition* using [Vosk toolkit](https://alphacephei.com/vosk/) ([Kaldi](http://kaldi-asr.org/) under the hood)
* *Task queue engine* using [Redis](https://redis.io/)
* *Full-text search* using [Elasticsearch Community Edition](https://www.elastic.co/downloads/elasticsearch-oss)
* *User Interface* using [React](https://reactjs.org/)

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

It takes a little while for Elastic to start up. Then, the OAS user interface and API are available at [`http://localhost:8080`](http://localhost:8080).

For the speech recognition to work, you'll need to download the models. Run this command once, it will download the models into the `./data/oas` volume:
```sh
docker-compose exec backend python download_models.py
```

Elastic Search wants quite a lot of free disc space. If the threshold is not met, it refuses to do anything. Run the script at `oas_worker/scripts/elastic-disable-threshold.sh` to disable the disc threshold (does not persist across Elastic restarts):
``` sh
docker-compose exec backend bash scripts/elastic-disable-threshold.sh
```

## Run locally for developing

To develop locally you may want to run OAS without Docker. You should install the following requirements beforehand:
- Python 3 and [poetry](https://python-poetry.org/docs/)
- For building the frontend: [Node.js](https://nodejs.org/en/) and npm or yarn
- [ffmpeg](https://www.ffmpeg.org/)

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

*Prepare and build backend*
```sh
cd oas_worker
poetry install
```

*Start elasticsearch and redis via Docker*
```sh
docker-compose -f docker-compose.dev.yml up
```

*Start OAS server and worker*
```sh
cd oas_worker
poetry run python server.py
# in another terminal:
poetry run celery -A app.tasks.tasks worker --loglevel=INFO
```

Open the UI in a browser at [http://localhost:8080](http://localhost:8080/).

### Development tips and tricks

Have a look at the [development guide](./docs/development.md).

## Configuration

OAS is configured through environment variables. If a `.env` file is present in the directory from which oas_worker is started the variables from there will be used. To customize the configuration, copy [`.env.default`](`oas_worker/.env.default`) in the `oas_worker` folder to `.env` and adjust the values.

|variable|default|description|
|-|-|-|
|`STORAGE_PATH`|`./data/oas`|Storage path for models, cached files and other assets|
|`FRONTEND_PATH`|`./frontend/dist`|Path to the built frontend that will be served at `/`|
|`HOST`|`0.0.0.0`|Interface for the HTTP server to bind to|
|`PORT`|`8080`|Port for HTTP server to listen on|
|`REDIS_URL`|`redis://localhost:6379/0`|URL to Redis server|
|`ELASTIC_URL`|`http://localhost:9200/`|URL to Elastic server (trailing slash is required)|
|`ELASTIC_INDEX`|`oas`|Name of the Elastic Search index to be created and used|
|`OAS_DEV`||If set to `1`: Enable development mode (see [Development guide](./docs/development.md)|


## License
[AGPL v3](LICENSE)


## Documentation
The official documentation is hosted on tbd.


## Discussion and Development
Most development discussions take place on github in this repo. Further, tbd.


## Contributing to Open Audio Search

All contributions, bug reports, bug fixes, documentation improvements, enhancements, and ideas are welcome.

Code of conduct, tbd.
