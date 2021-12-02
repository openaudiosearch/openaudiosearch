import httpx
import json
import argparse

OAS_URL = 'http://admin:password@localhost:8080/api/v1'

def create_feed(url):
    url = f"{OAS_URL}/feed"
    body = {
        "url": url,
        "media_jobs": {},
        "post_jobs": {}
    }
    res = httpx.post(url, json=body)
    res = res.json()
    return res

def get_changes(token: str):
    url = f"{OAS_URL}/changes/durable/{token}"
    res = httpx.post(url)
    res = res.json()
    return res



if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='OAS NLP example scripts')
    # parser.add_argument('dataset', metavar='d', type=str, help='Path to devset CSV')
    args = parser.parse_args()

    # generate feed from devset
    # serve via http

    url = 'http://localhost:6650/rss.xml'
    res = create_feed(url)
    print('created feed', res)

    # wait for all jobs to be finished
    token = 'evaluate_devset'
    while True:
        changes = get_changes(token)
        print('changes', json.dumps(changes))
        
    # get all posts
    # save posts and/or transcripts to files
    # (( optionally (or other script) to run evaluations/comparisons with previous results ))
   
    print(f"Result:\n{res['nlp']}")
