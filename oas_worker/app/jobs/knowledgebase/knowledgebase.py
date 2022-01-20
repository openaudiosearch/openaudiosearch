from app.worker import worker
from spacy.kb import KnowledgeBase
from app.jobs.spacy_pipe import spacy_load
from app.jobs.spacy_pipe import spacy_model
from Levenshtein import distance

@worker.job(name="knowledgebase")
def knowledgebase(ctx, args):
    """Simple proof-of-concept integration of spacys knowledgebase

    Args:
        ctx (Context): The context object contains the worker ID, the current job and enables access to the core client
        args ({id: string}): post_id

    Returns:
        patches: json patch
    """
    post_id = args["post_id"]
    post = ctx.get(f"/post/{post_id}")
    guid = post["$meta"]["guid"]
    if post["nlp"] is None or post["nlp"]["ned"] is None:
        return {}
    nlp = spacy_load(spacy_model)
    vocab = nlp.vocab
    kb = KnowledgeBase(vocab=vocab, entity_vector_length=300)
    # pprint(post["nlp"]["ned"])
    for k,v in post["nlp"]["ned"].items():
        freq = v['count']
        for item in v['results']:
            if distance(k, item["title"]) <= 3:
                title = item["title"]
                description = item["description"]
                qid = item["pageprops"]["wikibase_item"]
                desc = nlp(description)
                desc_enc = desc.vector
                kb.add_entity(entity=qid, entity_vector=desc_enc, freq=freq)
                kb.add_alias(alias=title, entities=[qid], probabilities=[1])
    post["nlp"]["kb"] = True

    patch = [
        {"op": "replace", "path": "/nlp", "value":post["nlp"]},
    ]
    patches = { guid: patch }

    return {
        "patches": patches
    }

