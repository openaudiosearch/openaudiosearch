from app.worker import worker
import httpx
import spacy
import json

from qwikidata.entity import WikidataItem, WikidataLexeme, WikidataProperty
from qwikidata.linked_data_interface import get_entity_dict_from_api

@worker.job(name="naive_ned")
def naive_ned(ctx, args):
    post_id = args["post_id"]
    post = ctx.get(f"/post/{post_id}")
    guid = post["$meta"]["guid"]
    nlp_data = post["nlp"]
    # Nothing to do if no nlp
    if nlp_data is None:
        return {}
    result  =  {}
    for named_entity in nlp_data["ner"]:
        search_result = json.loads(get_candidates(named_entity[0]))
        
        for candidate in search_result['search']:
            if candidate["match"]["type"] == "label":
                ent = get_entity_dict_from_api(candidate["id"])
                res = WikidataItem(ent)
                result[named_entity[0]] = candidate
                print(candidate["id"], candidate["match"]["text"], res.get_description()) 
        print(20* "-")   
    
    

def get_candidates(query):
    r = httpx.get('https://www.wikidata.org/w/api.php?action=wbsearchentities&search={}&language=de&format=json'.format(query))
    return r.text