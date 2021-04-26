from datetime import datetime
from elasticsearch_dsl import Document, Date, Integer, Keyword, Text
from elasticsearch_dsl.connections import connections

connections.create_connection(hosts=['localhost'])

class AudioObject(Document):
    headline = Text(fields={'raw': Keyword()})
    identifier = Keyword()
    url = Keyword()
    contentUrl = Keyword()
    encodingFormat = Keyword()
    abstract = Text()
    description = Text(fields={'raw': Keyword()})
    creator = Text(fields={'raw': Keyword()})
    contributor= Text(fields={'raw': Keyword()})
    genre = Keyword()
    datePublished = Date()
    duration = Keyword() # TODO: change to float?
    inLanguage = Keyword()
    dateModified = Date()
    licence = Keyword()
    publisher = Text(fields={'raw': Keyword()})
    transcript = Text()

    class Index:
        name = 'audio_objects'
        settings = {
        }