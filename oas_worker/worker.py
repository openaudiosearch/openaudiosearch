import time

from app.worker import Worker
from app.bin import run

worker = Worker()

@worker.task("asr", default_concurrency=1)
def asr(ctx, args, opts):
    meta = {
        "model": "foo1"
    }
    ctx.set_progress(0.0, meta=meta)

    time.sleep(2)
    ctx.set_progress(20)
    time.sleep(2)
    ctx.set_progress(40)
    time.sleep(2)
    ctx.set_progress(60)
    
    media_id = args["media_id"]
    guid = "oas.Media_" + media_id
    media = ctx.get("/media/" + media_id)
    transcript = {
        "text": "foo bar baz " + media_id + " " + str(media["duration"]),
        "parts": [
            { "start": 0.2, "end": 1.0, "conf": 1.0, "word": "foo" },
            { "start": 0.2, "end": 1.0, "conf": 1.0, "word": "bar" },
            { "start": 0.2, "end": 1.0, "conf": 1.0, "word": "baz" },
        ]
    }
    patch = [
        { "op": "replace", "path": "/transcript", "value": transcript },
    ]
    patches = {
        guid: patch
    }
    return {
        "meta": {
            "asrfoo": "bar"
        },
        "patches": patches
    }

@worker.task("nlp", default_concurrency=2)
def nlp(ctx, args, opts):
    ctx.log.info("hello info nlp")
    #  print("nlp", ctx.worker_id, "job", ctx.job_id, "ARGS", args, opts)
    time.sleep(1)
    ctx.log.info("done")
    return {
        "meta": {
            "nlpfoo": "bazoo"
        }
    }

if __name__ == '__main__':
    run(worker)

