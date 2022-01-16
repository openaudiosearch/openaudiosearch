# docker-compose setup for testing

This is a docker-compose file with the required backend services (but not worker or core) that does not have any persistence. No volumes are mounted, so everything will be destroyed when the containers are destroyed.

The ports of CouchDB, Ocypod and Elasticsearch are exposed on localhost.

Use with:

```
docker-compose up -d
```

to start. To stop and remove all containers, run:

```
docker-compose rm -f -s
```
