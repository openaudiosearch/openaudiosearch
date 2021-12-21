from app.worker import worker
import httpx
import json

from qwikidata.entity import WikidataItem, WikidataLexeme, WikidataProperty
from qwikidata.linked_data_interface import get_entity_dict_from_api

def get_candidates(query):
    r = httpx.get('https://www.wikidata.org/w/api.php?action=wbsearchentities&search={}&language=de&format=json'.format(query))
    print(20 * "#")
    print(r)
    return r.text

@worker.job(name="naive_ned")
def naive_ned(ctx, args):
    """[summary]

    Args:
        ctx ([type]): [description]
        args ([type]): [description]

    Returns:
        patches: json patch
    """
    post_id = args["post_id"]
    post = ctx.get(f"/post/{post_id}")
    guid = post["$meta"]["guid"]
    # Nothing to do if no nlp
    if post["nlp"] is None or post["nlp"]["ner"] is None:
        return {}
    nlp_data = post["nlp"]
    result  =  {}
    for named_entity in nlp_data["ner"]:
        print(40 * '#')
        print(named_entity)
        candidates = get_candidates(named_entity[0])
        print(candidates)
        if candidates is None:
            return {}
        try:
            candidates = json.loads(candidates)
        except:
	        return {}
        for candidate in candidates['search']:
            if candidate["match"]["type"] == "label":
                ent = get_entity_dict_from_api(candidate["id"])
                res = WikidataItem(ent)
                result[named_entity[0]] = candidate
                print(candidate["id"], candidate["match"]["text"], res.get_description())  
    post["nlp"]["ned"] = result        

    patch = [
        {"op": "replace", "path": "/nlp", "value": nlp_data},
    ]
    patches = { guid: patch }

    return {
        "patches": patches
    }
