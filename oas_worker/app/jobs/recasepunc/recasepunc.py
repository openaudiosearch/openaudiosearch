import os
import datetime
from recasepunc import CasePuncPredictor
from recasepunc import WordpieceTokenizer
from recasepunc import Config

from app.worker import worker

@worker.job(name="recasepunc")
def recasepunc(ctx, args):
    media_id = args["media_id"]
    media = ctx.get(f"/media/{media_id}")

    guid = media["$meta"]["guid"]

    transcript = media["transcript"]
    # Nothing to do if no transcript
    if transcript is None:
        return {}

    text_orig = transcript["text"].strip()

    model_base_path = ctx.config.model_path
    model_path = os.path.join(model_base_path, ctx.config.recase_model)
    predictor = CasePuncPredictor(model_path, lang="de", device="cpu")
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
        "processed": True,
        "created": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "model": ctx.config.recase_model
    }

    patch = [
        {"op": "replace", "path": "/transcript", "value": transcript},
    ]
    patches = { guid: patch }

    return {
        "patches": patches
    }
