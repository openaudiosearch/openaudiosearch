# Notes on Elastic

We want to encode the ASR metadata for each word (start, end, conf) into the elastic index with the `delimited_payload` filter.

## Create an index with a delimited payload filter:

```json
{
  "mappings": {
    "properties": {
      "text": {
        "type": "text",
        "term_vector": "with_positions_payloads",
        "analyzer": "whitespace_plus_delimited"
      }
    }
  },
  "settings": {
    "analysis": {
      "analyzer": {
        "whitespace_plus_delimited": {
          "tokenizer": "whitespace",
          "filter": [ "plus_delimited" ]
        }
      },
      "filter": {
        "plus_delimited": {
          "type": "delimited_payload",
          "delimiter": "|",
          "encoding": "float"
        }
      }
    }
  }
}
'
```


top level "transcript" field,

token|mediaNum,start,end,conf

