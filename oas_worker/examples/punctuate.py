import argparse
import os
import httpx
import sys
from punctuator import Punctuator

sys.path.insert(0, "..")
from app.config import config


# Run with: poetry run python nlp.py <OAS-POST-ID>

OAS_URL = os.environ.get("OAS_URL") or "http://admin:password@localhost:8080/api/v1"
MODEL = "subs_norm1_filt_5M_tageschau_euparl_h256_lr0.02.pcl"
print(OAS_URL)

#  _punctuator = None
def get_punctuator():
    model_path = config.local_dir(os.path.join("models", "punctuator2", MODEL))
    print(model_path)
    punctuator = Punctuator(model_path)
    return punctuator


def patch_post(post_id, patch):
    url = f"{OAS_URL}/post/{post_id}"
    res = httpx.patch(url, json=patch)
    res = res.json()
    return res

def get_post(post_id):
    url = f"{OAS_URL}/post/{post_id}"
    res = httpx.get(url)
    res = res.json()
    return res

def transcript_to_text(transcript):
    text = ""
    if not transcript or not len(transcript["parts"]):
        return text
    for part in transcript["parts"]:
        text = text + " " + part["word"]
    return text


def punctuate_text(text):
    punctuator = get_punctuator()
    return punctuator.punctuate(text)

def punctuate_transcript(transcript):
    text = transcript_to_text(transcript)
    print("text", text)
    result = punctuate_text(text)
    print("result", result)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='OAS NLP example scripts')
    parser.add_argument('post_id', metavar='P', type=str, help='Post ID to process')
    args = parser.parse_args()

    # Get post
    post_id = args.post_id
    print("Post ID: " + post_id)
    post = get_post(post_id)
    # print("Post: ")
    # print(post)

    for media in post["media"]:
        result = punctuate_transcript(media["transcript"])

    # Output nlp stuff
    #  patch = [
    #      { "op": "add", "path": "/nlp", "value": nlp_result },
    #  ]
    #  patch_post(post_id, patch)

    #  res = get_post(post_id)
    #  print(f"Result:\n{res['nlp']}")
