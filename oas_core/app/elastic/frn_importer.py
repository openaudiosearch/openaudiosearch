import feedparser
import json
from elasticsearch import Elasticsearch
from pprint import pprint
# from functools import filter

elastic_url: str = 'http://localhost:9200/'
elastic_index: str = 'oas_feed2'
connection = Elasticsearch([elastic_url])


Feed = feedparser.parse("https://www.freie-radios.net/portal/podcast.php?rss")
results = []

def put(connection, doc, id):
    doc = json.dumps(doc)
    res = connection.index(index=elastic_index, id=id, body=doc, doc_type="_doc")
    return res

def put_feeds():
    for id, entry in enumerate(Feed.entries):
        mapping = {
            'headline': entry.title,
            'identifier': entry.id,
            'url': entry.link,
            'contentUrl': [link.href for link in entry.links if link.type.startswith('audio')][0],
            'encodingFormat': [link.type for link in entry.links if link.type != 'text/html'][0],
            'abstract': entry.subtitle,
            'description': entry.summary,
            'creator': [entry.author],
            'contributor': list(set([author.name for author in entry.authors])),
            'genre': entry.frn_art,
            'datePublished': [entry.published], #time.struct_time-object TODO: change to date format
            'duration': entry.frn_laenge,
            'inLanguage': [entry.frn_language],
            'dateModified': entry.frn_last_update,
            'licence': entry.frn_licence,
            'publisher': entry.frn_radio, # TODO: check if publisher is correct category
            # TODO: frn_serie refers to the number of a radio series. Needs relations implemented
        }
        # results.append(mapping)
        put(connection, mapping, id)

if name == "__main__":
    put_feeds()
    # print(results)
    # pprint(mapping)
    # pprint([entry.authors for entry in Feed.entries])