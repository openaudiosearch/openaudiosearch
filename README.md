# Open Audio Search - Rust core

*Work in progress*

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
