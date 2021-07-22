# Open Audio Search - Rust core

<a href="https://openaudiosearch.github.io/openaudiosearch/doc/oas_core/">
  <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
    alt="Rust docs" />
</a>

License: AGPL v3

*Work in progress*

Open Audio Search uses automatic speech recognition to extract text from audio, which is then indexed in a search engine to enable recommendation of similiar broadcasts to users.
With Open Audio Search, we want to make the archives of community media, radio stations, podcasts searchable and discoverable, through open source tech and APIs.

This is the new core for Open Audio Search written in [Rust](https://www.rust-lang.org/).

## Getting started

Install [Rust](https://rust-lang.org) with [Rustup](https://rustup.rs/).

Install [Docker](https://www.docker.com/) and [Docker Compose](https://docs.docker.com/compose/)

### Build and run OAS

```
cargo run
```

You can increase the log level by running oas with `RUST_LOG=debug cargo run`.

The CLI has a built-in help (TODO: Document commands here in the README once it's a bit more settled).

Most commands need a running instance of Elasticsearch and CouchDB. A development setup can be started in this folder with:

```
docker-compose up`
```
