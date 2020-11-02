

def search_elastic(self, search_term):
    search_param = {"query": {
        "match": {
            "text": search_term
        }
    }}
    response = self.es.search(index=self.index_name, body=search_param, doc_type="_doc")
    return response
