from datetime import datetime
from elasticsearch import Elasticsearch
from pprint import pprint
#from app.core.job import Worker
import json

#from app.config import config
config = {"es_host": "localhost",
            "es_port": "9200",
            "index_name": "oas"}


class SearchIndex:
    instance = None
    def __init__(self, host=config["es_host"],
                    port=config["es_port"],
                    index_name=config["index_name"],
                    ssl=False,
                    check_certs=False,
                    certs=""):
        if not SearchIndex.instance:
            SearchIndex.instance = SearchIndex.__SearchIndex(host,
                port,
                index_name,
                ssl,
                check_certs,
                certs)
        else:
            SearchIndex.instance.host = host
            SearchIndex.instance.port = port
            SearchIndex.instance.index_name = index_name
            SearchIndex.instance.ssl = ssl
            SearchIndex.instance.check_certs = check_certs
            SearchIndex.instance.certs = certs
    
    def __getattr__(self, name):
        return getattr(self.instance, name)

    class __SearchIndex:
        def __init__(self,
                    host,
                    port,
                    index_name,
                    ssl,
                    check_certs,
                    certs):
            self.connection = Elasticsearch([host + ":"+port])
            self.index_name = index_name
            self.certs = certs
            self.ssl = ssl
            self.index = self.connection.indices.create(index=index_name, ignore=400)

        def is_connected(self, host, port):
            if self.connection != host + ":" + port:
                return False
            else:
                return True

        def put(self, doc):
            doc = json.dumps(doc.reprJSON(), cls=Encoder)
            res = self.connection.index(index=self.index_name, id=id, body=doc, doc_type="_doc")
            return res

        def get(self, id):
            self.connection.get(index=self.index_name, id=id, doc_type="_doc")
        
        def get_con(self):
            return self.connection

        def search(self, search_term):
            search_param = {"query": {
                "match": {
                "text": search_term
                }
            }}
            response = self.connection.search(index=self.index_name, body=search_param, doc_type="_doc")
            return response
        
        def refresh(self):
            self.connection.indices.refresh(index=self.index_name)


class Document:
    def __init__(self, asr_result, path_to_audio="to-do.mp3"):
        self.results = []
        for part in asr_result["parts"]:
            for word_result in part["result"]:
                res = AsrInnerResult(
                    word_result["conf"], word_result["start"], word_result["end"], word_result["word"])
                self.results.append(res)
        self.text = asr_result["text"]
        self.path_to_audio = path_to_audio
        self.created_at = datetime.now()
       


    def reprJSON(self):        
        return dict(results=[result.reprJSON() for result in self.results],
                    text=self.text,
                    path_to_audio=self.path_to_audio,
                    created_at=self.created_at
                    )


class AsrInnerResult():
    def __init__(self, conf, start, end, word):
        self.conf = conf
        self.start = start
        self.end = end
        self.word = word

    def reprJSON(self):
        return dict(conf=self.conf,
                    start=self.start,
                    end=self.end,
                    word=self.word
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
    # asr_result = {"result": [{
    #     "conf": 0.49457,
    #     "end": 2.34,
    #     "start": 1.35,
    #     "word": "hello"},
    #     {
    #     "conf": 0.9,
    #     "end": 2.3,
    #     "start": 1.4,
    #     "word": "hello"}],
    #     "text": "transcript"}
    asr_result = {'text': ' fünfundzwanzig jahren leipziger geschichte mit perspektiven als waren zum beispiel erinnerung es kommt',
                  'parts': [
                      {'result': [
                          {'conf': 0.983098, 'end': 1.41, 'start': 0.27, 'word': 'fünfundzwanzig'},
                          {'conf': 0.42963, 'end': 2.034147, 'start': 1.41, 'word': 'jahren'},
                          {'conf': 0.956407, 'end': 2.669977, 'start': 2.1, 'word': 'leipziger'},
                          {'conf': 0.997978, 'end': 3.27, 'start': 2.67082, 'word': 'geschichte'},
                          {'conf': 0.674975, 'end': 3.48, 'start': 3.33, 'word': 'mit'},
                          {'conf': 0.998419, 'end': 4.5, 'start': 3.48, 'word': 'perspektiven'},
                          {'conf': 1.0, 'end': 5.5798, 'start': 5.34, 'word': 'als'},
                          {'conf': 0.829022, 'end': 5.97, 'start': 5.58, 'word': 'waren'},
                          {'conf': 1.0, 'end': 6.48, 'start': 6.33, 'word': 'zum'},
                          {'conf': 1.0, 'end': 6.904158, 'start': 6.48, 'word': 'beispiel'},
                          {'conf': 0.678397, 'end': 7.411906, 'start': 6.904158, 'word': 'erinnerung'},
                          {'conf': 0.898544, 'end': 7.92, 'start': 7.74, 'word': 'es'},
                          {'conf': 0.999274, 'end': 8.43, 'start': 7.920014, 'word': 'kommt'}
                      ],
                          'text': 'fünfundzwanzig jahren leipziger geschichte mit perspektiven als waren zum beispiel erinnerung es kommt'}
                  ]}

    path_to_audio = "path/to/audio"
    
    search_index = SearchIndex("localhost", "9200", "oas")
    doc = Document(asr_result, path_to_audio)

    pprint(search_index.put(doc))

