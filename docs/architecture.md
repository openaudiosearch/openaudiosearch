# Architecture

The following paragraphs will outline the technical architecture of Open Audio Search (*OAS*). This is directed towards developers and system administrators. We intend to expand and improve this document over time. If something in here is unclear to you or you miss explanations, please feel invited to [open an issue](https://github.com/openaudiosearch/openaudiosearch/issues/new/choose).

### Core (or backend)

This is a server daemon written in [Rust](https://rust-lang.org). It provides a REST-style HTTP API and talks to our main data services: [CouchDB](https://couchdb.org) and [Elasticsearch](https://www.elastic.co/) or [OpenSearch](https://opensearch.org/).

The core compiles to a static binary that includes various sub commands, the most important being the `run` command which runs all parts of the core concurrently. The other commands currently mostly serve debug and administration purposes.

The core oftenly uses the [`_changes`](https://docs.couchdb.org/en/stable/api/database/changes.html) endpoint in CouchDB. This endpoint returns a live-streaming list of all changes to the CouchDB. Internally, CouchDB maintains a log of all changes made to the database, and each revision is assigned a sequence string. Various services in OAS make use of this feature to visit all changes made to the database.

The core uses the asynchronous [Tokio](https://tokio.rs/) runtime to run various tasks in parallel. Currently, this includes:

- A **HTTP server** to provide a REST-style API that allows to `GET`, `POST`, `PUT` and `PATCH` the various data records in OAS (feeds, posts, medias, transcripts, ...). It also manages authentication for the routes. It can serve the web frontend either by statically including the HTML, JavaScript and other assets directly in the binary, or by proxying to another HTTP server (useful for development). The HTTP server uses [Rocket](https://rocket.rs/), an async HTTP framework for Rust.
- An **indexer** service that listens on the CouchDB changes stream and indexes all posts, medias and transcripts into an Elasticsearch index. For the index, our data model is partially flattened to make querying more straightforward.
- The **RSS importer** also listens on the changes stream for *Feed* records and then periodically fetches these RSS feeds and saves new items into *Post* and *Media* records. It also sets a flag on the *Media* records depending on the settings that are part of the *Feed* record whether a transcribe job is wanted or not.
- A **job queue** also listens on the changes stream and may, depending on a *TaskState* flag, create jobs for the worker. The job services currently uses the [Celery](https://docs.celeryproject.org/en/stable/getting-started/introduction.html) job queue with a [Redis](https://redis.io/) backend.

The core still is rough at several edges. While it works, the internal APIs will still change quite significantly towards better abstractions that makes these data pipelines more flexible and reliable. We need better error handling in cases of failures and better observability. There is *a lot* of room for optimizations. For example, at this point each service consumes a separate changes stream, and there is no internal caching of data records. This also means that any performance issues that might be visible at the moment will have a clear path to being solved.

### Worker

The worker is written in Python. It currently uses the [Celery](https://docs.celeryproject.org/en/stable/) job queue to retrieve jobs that are created in the core. It performs the jobs and then posts back its results to the core over the HTTP API exposed by the core. Usually, it will send a set of JSON patches to update one or more records in the database with its results.

Currently, the two main tasks are:

* `transcribe`: This task takes an audio file, downloads and converts it into a WAV file and then uses the [Vosk](https://alphacephei.com/vosk/) toolkit to create a text transcription of the audio file. Vosk is based on [Kaldi ASR](https://kaldi-asr.org/), an open-source speech-to-text engine. To create these transcripts, a model for the language of the audio is needed. At the moment, the only model that is automatically used in OAS is the German language model from the Vosk model repository. We will soon provide more models, and will then also need to implement a mechanism to first detect the spoken language to then use the correct model.
* `nlp`: This task takes the transcript, description and other metadata of a post as input, and then performs various NLP (natural language processing) steps on this text. Most importantly, it tries to extract keywords through an NER (named entity recognition) pipeline. Currently, we are using the [SpaCy](https://spacy.io/) toolkit for this task.

We plan to add further processing tasks, e.g. to detect the language of speech, restore punctuation in the transcript, chunk the transcript into fitting snippets for subtitles.

### Frontend

The frontend is a single-page web application written with [React](https://reactjs.org/). It uses the [Chakra UI](https://chakra-ui.com/) toolkit for various components and UI elements. The frontend talks to the core through its HTTP API. It is mostly public-facing with a dynamic search page that allows filtering and faceting the search results. We currently use [ReactiveSearch](https://github.com/appbaseio/reactivesearch) components for the search page. It also features a login form for administrators, which unlocks administrative sections. Currently, this only includes a page to manage RSS feeds and some debug sections. We will add more administrative features in the future.

### Packaging

OAS includes [Dockerfiles](https://docs.docker.com/engine/reference/builder/) for the core and the worker to easily package and run OAS as Linux containers. It also includes [docker-compose](https://docs.docker.com/compose/) files to easily start and run OAS together with all required services: CouchDB, Elasticsearch and Redis.

The docker images can be built from source with the provided Dockerfiles. We also push nightly images to Dockerhub, which allows to run OAS without building from source.
