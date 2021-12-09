from recasepunc import CasePuncPredictor
from recasepunc import WordpieceTokenizer
from recasepunc import Config

from app.worker import worker

@worker.job(name="recasepunc")
def recasepunc(ctx, args):
    post_id = args["post_id"]
    post = ctx.get(f"/post/{post_id}")
    guid = post["$meta"]["guid"]
    # TODO: abort if no transcript
    text = post["media"][0]["transcript"]["text"]
    model_base_path = config.model_path
    model_path = os.path.join(model_base_path, config.recase_model)
    predictor = CasePuncPredictor(model_path, lang="de")
    tokens = list(enumerate(predictor.tokenize(text)))
    results = ""

    for token, case_label, punc_label in predictor.predict(tokens, lambda x: x[1]):
        prediction = predictor.map_punc_label(predictor.map_case_label(token[1], case_label), punc_label)
        if token[1][0] != '#':
            results = results + ' ' + prediction
        else:
            results = results + prediction

    patch = [
        {"op": "replace", "path": "/transcript", "value": results},
    ]
    patches = { guid: patch }
    return {
        "patches": patches
    }
