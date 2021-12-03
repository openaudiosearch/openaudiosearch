from app.worker import worker

@worker.job(name="example")
def example_job(ctx, args):
    print(args)
    identifier = args["identifier"]
    query = {
        "query": { 
            "term": {
                "identifier": identifier
            }
        },
        "_source": False,
        "fields": [
            "$meta.id"
        ]
    }
    res = ctx.post("/search/oas/_search", body=query)
    print("res", res)
    return {}
