from datetime import datetime
from elasticsearch_dsl import Document, Date, Integer, Keyword, Text
from elasticsearch_dsl.connections import connections
import requests
import time

from app.elastic.configs import index_configs
from app.config import config

connections.create_connection(hosts=[config.elastic_url])


class AudioObject(Document):
    headline = Text(fields={'raw': Keyword()})
    identifier = Keyword()
    url = Keyword()
    contentUrl = Keyword()
    encodingFormat = Keyword()
    abstract = Text()
    description = Text(fields={'raw': Keyword()})
    creator = Text(fields={'raw': Keyword()})
    contributor = Text(fields={'raw': Keyword()})
    genre = Keyword()
    datePublished = Date()
    duration = Keyword()  # TODO: change to float?
    inLanguage = Keyword()
    dateModified = Date()
    licence = Keyword()
    publisher = Text(fields={'raw': Keyword()})
    transcript = Text()

    class Index:
        name = config.elastic_index
        settings = {
        }

    @classmethod
    def get_keys(cls):
        return list(cls.__dict__["_doc_type"].__dict__["mapping"].to_dict()["properties"].keys())


def wait_for_elastic():
    url = config.elastic_url + '_cat/health'
    while True:
        try:
            res = requests.get(url)
            return
        except Exception as e:
            print(f'Elastic cannot be reached at {url}, retrying in 1 second')
            time.sleep(1)
