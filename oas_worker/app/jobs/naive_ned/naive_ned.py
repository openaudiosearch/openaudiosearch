from app.worker import worker
from enum import Enum
from httpx import Client
from pprint import pprint
import json

@worker.job(name="naive_ned")
def naive_ned(ctx, args):
    """Simple implementation of naive named entity linking with Wikidata.
       It simply queries the Wikibase API and takes the first three results.
       https://www.wikidata.org/w/api.php?action=help&modules=wbgetentities
       https://www.wikidata.org/w/api.php?action=wbgetentities&ids=Q1%7CQ42&props=descriptions&languages=en%7Cde%7Cfr

    Args:
        ctx (Context): The context object contains the worker ID, the current job and enables access to the core client
        args ({id: string}): post_id

    Returns:
        patches: json patch
    """
    post_id = args["post_id"]

    post = ctx.get(f"/post/{post_id}")
    guid = post["$meta"]["guid"]
    if post["nlp"] is None or post["nlp"]["ner"] is None:
        return {}
    accepted_labels = ["LOC", "PER"]
    result = dict()
    for item in post["nlp"]["ner"]:
        text, label, _start, _end = item
        text = text.casefold()
        pages = None
        if text in result:
            result[text] = {
                "results":  result[text]["results"],
                "count": result[text]["count"] + 1
            }
        else:
            if label in accepted_labels:
               params = set_search_params(text)
               r = search_wikipedia(params, language="de")
               if r:
                   pages = r.get("pages")
               if pages:
                   result[text] = {
                    "results":  sorted(list(pages.values()),key=lambda x :x['index']),
                    "count": 1
                    }
            
    post["nlp"]["ned"] = result        
    
    patch = [
        {"op": "replace", "path": "/nlp", "value":post["nlp"]},
    ]
    patches = { guid: patch }

    return {
        "patches": patches
    }

class lang(Enum):
    de = "de"
    en = "en"
    fr = "fr"
    es = "es"

    @classmethod
    def has_value(cls, value):
        return value in cls._value2member_map_


def set_search_params(query, limit = 3) :
    params = {
    "action": "query",
    "format": "json",
    "prop":"pageprops|description",
    "ppprop":"wikibase_item",
    "generator" : "search",
    "gsrsearch": "\"{}\"".format(query),
    "gsrlimit": limit,
    "redirects": True,
    }
    return params

def search_wikipedia(params, language = "de"):
    if lang.has_value(language):
        headers = {"user-agent": "openaudiosearch/0.0.1"}
        with Client(headers=headers) as client:
            url = "https://{}.wikipedia.org/w/api.php".format(language)
            r = client.get(url, params=params)
            result = r.json()
            error = result.get("error")
            warnings = result.get("warnings")
            if not warnings and not error and result.get("query"):
                return result["query"]
            elif error:
                print(error) 
            elif warnings:
                print(warnings) 
            return


