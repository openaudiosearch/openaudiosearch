import os
import time
import datetime
from recasepunc import CasePuncPredictor
from recasepunc import WordpieceTokenizer
from recasepunc import Config

from app.worker import worker

def perform_recase(config, transcript):
    # Nothing to do if no transcript
    if transcript is None:
        return {}

    timer = time.time()
    text_orig = transcript["text"].strip()

    model_base_path = config.model_path
    model_path = os.path.join(model_base_path, config.recase_model)
    predictor = CasePuncPredictor(model_path, lang="de")
    tokens = list(enumerate(predictor.tokenize(text_orig)))

    text_recased = ""
    for token, case_label, punc_label in predictor.predict(tokens, lambda x: x[1]):
        prediction = predictor.map_punc_label(predictor.map_case_label(token[1], case_label), punc_label)
        # the prediction token start with # if they are within a word
        if token[1][0] != '#':
            prediction = " " + prediction

        text_recased = text_recased + prediction

    recased_tokens = text_recased.split()
    i = 0
    puncts = ["," ,"!" ,"." ,"?"]
    for i in range(len(transcript["parts"])):
        part = transcript["parts"][i]

        token = recased_tokens[i]
        suffix = None
        word = token
        if token[-1] in puncts:
            suffix = token[-1]
            word = token[:-1]

        part["word"] = word
        part["suffix"] = suffix

    transcript["text"] = text_recased
    transcript["meta"]["recasepunc"] = {
        "created": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "model": config.recase_model,
        "time": round(time.time() - timer, 2)
    }
    return transcript

@worker.job(name="recasepunc")
def recasepunc(ctx, args):
    media_id = args["media_id"]
    media = ctx.get(f"/media/{media_id}")

    guid = media["$meta"]["guid"]

    transcript = media["transcript"]
    updated_transcript = perform_recase(ctx.config, transcript)

    patch = [
        {"op": "replace", "path": "/transcript", "value": updated_transcript},
    ]
    patches = { guid: patch }

    return {
        "patches": patches
    }
