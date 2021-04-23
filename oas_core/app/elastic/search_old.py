from datetime import datetime
from elasticsearch import Elasticsearch
from pprint import pprint
import requests
import time
import json
#from app.core.job import Worker

from configs.index_configs import index_configs
# from app.config import config

elastic_url: str = 'http://localhost:9200/'
elastic_index: str = 'oas_feed2'

index_confs = index_configs()

def wait_for_elastic():
    url = config.elastic_url + '_cat/health'
    while True:
        try:
            res = requests.get(url)
            return
        except Exception as e:
            print(f'Elastic cannot be reached at {url}, retrying in 1 second')
            time.sleep(1)


class SearchIndex:
    instance = None
    def __init__(self, elastic_url=elastic_url,
                    index_name=elastic_index,
                    ssl=False,
                    check_certs=False,
                    certs="",
                    delete_old_index=False,
                    index_config='prefix'):
        if delete_old_index or not SearchIndex.instance:
            SearchIndex.instance = SearchIndex.__SearchIndex(elastic_url,
                index_name,
                ssl,
                check_certs,
                certs,
                index_config)
        else:
            SearchIndex.instance.elastic_url = elastic_url
            SearchIndex.instance.index_name = index_name
            SearchIndex.instance.ssl = ssl
            SearchIndex.instance.check_certs = check_certs
            SearchIndex.instance.certs = certs
    
    def __getattr__(self, name):
        return getattr(self.instance, name)

    class __SearchIndex:
        def __init__(self,
                    elastic_url,
                    index_name,
                    ssl,
                    check_certs,
                    certs,
                    index_config):
            self.connection = Elasticsearch([elastic_url])
            self.index_name = index_name
            self.certs = certs
            self.ssl = ssl
            self.index_config = index_config
            self.index = self.connection.indices.create(
                index=index_name,
                body=index_confs[self.index_config], 
                ignore=400)

        def is_connected(self, host, port):
            if self.connection != host + ":" + port:
                return False
            else:
                return True

        def put(self, doc, id):
            doc = json.dumps(doc.reprJSON(), cls=Encoder)
            res = self.connection.index(index=self.index_name, id=id, body=doc, doc_type="_doc")
            return res

        def get(self, id):
            self.connection.get(index=self.index_name, id=id, doc_type="_doc")
        
        def get_con(self):
            return self.connection

        def search(self, search_term, range_queries=None):
            search_param = \
                {"query": {
                    "bool": {
                        "should": [
                            {"match": {"description": {"query": search_term,
                                                "operator": "and"}}},
                            {"match": {"headline": {"query": search_term}}}
                        ]
                    }
                },
                "aggs": {
                    "publisher_agg": {
                        "terms": {
                            "field": "publisher.keyword",
                            "size": 10
                        }
                    }
                }
            }
            
                        #     {"query": {
            #         "bool": {
            #             "should": [
            #                 {"match": {"description": {"query": search_term,
            #                                     "operator": "and"}}},
            #                 {"match": {"headline": {"query": search_term}}}
            #             ]
            #         # },
            #         # "range": {
            #         #     "date_production": {
            #         #         "gte": '2020-04-01',
            #         #         "lte": '2021-05-02'
            #         #     }
            #         }
            #     }
            # }
            response = self.connection.search(index=self.index_name, body=search_param, doc_type="_doc")
            return response
        
        def refresh(self):
            self.connection.indices.refresh(index=self.index_name)


class Document:
    def __init__(self, feed_entry):
        self.headline = feed_entry['headline']
        self.identifier = feed_entry['identifier']
        self.url = feed_entry['url']
        self.contentUrl = feed_entry['contentUrl']
        self.encodingFormat = feed_entry['encodingFormat']
        self.abstract = feed_entry['abstract']
        self.description = feed_entry['description']
        self.creator = feed_entry['creator']
        self.contributor = feed_entry['contributor']
        self.genre = feed_entry['genre']
        self.datePublished = feed_entry['datePublished']
        self.duration = feed_entry['duration']
        self.inLanguage = feed_entry['inLanguage']
        self.dateModified = feed_entry['dateModified']
        self.licence = feed_entry['licence']
        self.publisher = feed_entry['publisher']


    def reprJSON(self):        
        return dict(headline=self.headline,
            identifier = self.identifier,
            url = self.url,
            contentUrl = self.contentUrl,
            encodingFormat = self.encodingFormat,
            abstract = self.abstract,
            description = self.description,
            creator = self.creator,
            contributor = self.contributor,
            genre = self.genre,
            datePublished = self.datePublished,
            duration = self.duration,
            inLanguage = self.inLanguage,
            dateModified = self.dateModified,
            licence = self.licence,
            publisher = self.publisher
        )


class Encoder(json.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, datetime):
            return obj.__str__()
        elif hasattr(obj, 'reprJSON'):
            return obj.reprJSON()
        else:
            return json.JSONEncoder.default(self, obj)


if __name__ == "__main__":

    # feed_entry = {
    #     'abstract': 'Tobias Pfüger, MdB die Linke, berichtet aus dem "Verteidigungs"ausschuss des Bundestags am 21.April',
    #     'contentUrl': 'https://www.freie-radios.net/mp3/20210421-abzugderbund-108544.mp3',
    #     'contributor': ['Reinhard grenzenlos (bermuda.funk - Freies Radio Rhein-Neckar)'],
    #     'creator': ['Reinhard grenzenlos (bermuda.funk - Freies Radio Rhein-Neckar)'],
    #     'dateModified': 'Wed, 21 Apr 2021 16:22:58 +0200',
    #     'datePublished': ['Wed, 21 Apr 2021 16:22:58 +0200'],
    #     'description': 'Tobias Pfüger, MdB die Linke, berichtet aus dem "Verteidigungs"ausschuss des Bundestags am 21.April 2021',
    #     'duration': '3:90',
    #     'encodingFormat': 'audio/mpeg',
    #     'genre': 'Reportage',
    #     'headline': 'Abzug der Bundeswehr aus Afghanistan (Serie 323: Grenzenlos)',
    #     'identifier': 'https://www.freie-radios.net/108544',
    #     'inLanguage': ['deutsch'],
    #     'licence': 'by-nc-sa',
    #     'publisher': 'bermuda.funk - Freies Radio Rhein-Neckar',
    #     'url': 'https://www.freie-radios.net/108544'
    # }
    
    search_index = SearchIndex(delete_old_index=False)
    # doc = Document(feed_entry)
    # #PUT Document in index
    # pprint("INDEX")
    # pprint(search_index.put(doc, "1"))
    pprint("SEARCH")
    pprint(search_index.search("polizei"))
