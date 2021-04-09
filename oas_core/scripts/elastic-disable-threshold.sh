#/bin/sh

ELASTIC_URL="${ELASTIC_URL:-"http://localhost:9200/"}"

curl -XPUT -H "Content-Type: application/json" ${ELASTIC_URL}_cluster/settings -d '{ "transient": { "cluster.routing.allocation.disk.threshold_enabled": false } }'
curl -XPUT -H "Content-Type: application/json" ${ELASTIC_URL}_all/_settings -d '{"index.blocks.read_only_allow_delete": null}'
