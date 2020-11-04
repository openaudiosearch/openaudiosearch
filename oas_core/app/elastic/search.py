from datetime import datetime
from elasticsearch import Elasticsearch
from pprint import pprint
from app.core.job import Worker
import json

from app.config import config

es = None
class SearchIndex():

    def __init__(self,
                 host=config.es_host,
                 port=config.es_port,
                 index_name=config.index_name,
                 ssl=False,
                 check_certs=False,
                 certs=""):
        global es
        if not es:
            es = self
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


class Document():
    def __init__(self, asr_result, path_to_audio="to-do.mp3"):
        self.results = []
        for result in asr_result["result"]:
            result = AsrInnerResult(
                result["conf"], result["start"], result["end"], result["word"])
            self.results.append(result)
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
    asr_result = {"result": [{
        "conf": 0.49457,
        "end": 2.34,
        "start": 1.35,
        "word": "hello"},
        {
        "conf": 0.9,
        "end": 2.3,
        "start": 1.4,
        "word": "hello"}],
        "text": "transcript"}

    path_to_audio = "path/to/audio"
    
    search_index = SearchIndex("localhost", "9200", "oas")
    doc = Document(asr_result, path_to_audio)

    pprint(search_index.put(doc))

