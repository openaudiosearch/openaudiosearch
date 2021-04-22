def index_configs():
  return {"prefix": {
  "settings": {
    "analysis": {
      "analyzer": {
        "autocomplete": {
          "tokenizer": "autocomplete",
          "filter": [
            "lowercase"
          ]
        },
        "autocomplete_search": {
          "tokenizer": "lowercase"
        }
      },
      "tokenizer": {
        "autocomplete": {
          "type": "edge_ngram",
          "min_gram": 2,
          "max_gram": 10,
          "token_chars": [
            "letter"
          ]
        },
        "keyword_tokenizer": {
          "type": "custom",
          "filter": [
          "lowercase",
          "asciifolding"
          ],
          "tokenizer": "keyword"
          }
      }
    }
  },
  "mappings": {
    "properties": {
      "description": {
        "type": "text",
        "analyzer": "autocomplete",
        "search_analyzer": "autocomplete_search"
      },
      "headline": {
        "type": "text",
        "analyzer": "autocomplete",
        "search_analyzer": "autocomplete_search"
      },
      "publisher": {
        "type": "keyword",
        "analyzer": "keyword_tokenizer"
      }
      # "date_production": {
      #   "type": "date",
      #   "format": "YYYY-MM-dd HH:mm:ss||YYY-MM-dd"
      
    }
  }
}
}