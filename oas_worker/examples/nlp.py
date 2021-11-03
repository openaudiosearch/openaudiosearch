import argparse
import httpx
import sys

sys.path.insert(0, "..")
from app.tasks.spacy_pipe import SpacyPipe


# Run with: poetry run python nlp.py <OAS-POST-ID>

OAS_URL = 'http://admin:password@localhost:8080/api/v1'

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


def nlp(post):
    text = post["description"]

    spacy = SpacyPipe(["ner", "textrank"])
    res = spacy.run(text)

    # debug
    print(f"Text:\n{text}")
    print(f"NLP Result:\n{res}")

    return res


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

    # Do NLP stuff
    nlp_result = nlp(post)
    # nlp_result = { "keywords": ["foo", "bazoo"] }

    # Output nlp stuff
    patch = [
        { "op": "add", "path": "/nlp", "value": nlp_result },
    ]
    patch_post(post_id, patch)

    res = get_post(post_id)
    print(f"Result:\n{res['nlp']}")
