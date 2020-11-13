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


## Installation and Usage

Follow the [installation guide](./docs/install.md) to install all dependencies.

#### Configuration

OAS is configured through an `.env` file in the directory from where you invoke it. To customize the configuration, copy [`.env.default`](`oas_core/.env.default`) in the `oas_core` folder to `.env` and adjust the values.

By default, all data is stored in `/tmp/oas`. To keep models, downloads and intermediate results change the `STORAGE_PATH` setting to a non-temporary path.


#### Run Demo

Build and start frontend:
```
cd frontend
yarn
yarn build
yarn start
```

Start server:
```
cd oas_core
python server.py
```

Start worker:
```
cd oas_core
python worker.py
```

Open demo in browser at [http://localhost:8080/ui/index.html](http://localhost:8080/ui/index.html)


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
