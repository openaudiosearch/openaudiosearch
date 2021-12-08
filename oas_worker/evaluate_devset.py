#!/usr/bin/env python
""" evaluate_devset

Evaluate OAS Devset:
  * Keywords
  * Transcripts (TBD)

Author(s): flipsimon, datadonk23
Date: 08.12.21
"""

import httpx
import json
import argparse
import pandas as pd

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


def get_true_labels(fpath: str):
    """
    Gets ground truth labels from devset spreadsheet and returns them as a
    dict with track ID as key and keywords list as values.

    :param fpath: Filepath to devset spreadsheet
    :return: List of Dicts {cba ID: keywords list}
    :return list: list(dict(cba ID: [keywords]))
    """
    ground_truth = []
    df = pd.read_csv(devset_fpath)
    df["cba_id"] = df["URL"].apply(
        lambda s: "https://cba.fro.at/?p=" + s.split("/")[-1]
    )
    df["keywords"] = df.OAS_tags.str.split(",")
    for _, row in df.iterrows():
        ground_truth.append({row["cba_id"]: row["keywords"]})

    return ground_truth


def devsetIDs_to_cbaIDs(fpath: str):
    """
    Transforms IDs from devset spreadsheet to CBA IDs in OAS posts
    (identifier field, not post ID!).

    :param fpath: Filepath to devset spreadsheet
    :return: List of OAS identifiers
    """
    df = pd.read_csv(devset_fpath)
    cba_head = "https://cba.fro.at/"
    oas_param = "?p="
    ids = df.URL.apply(lambda s: cba_head + oas_param + s.split("/")[-1])

    return ids.to_list()


def get_post_ids(cba_ids: list):
    """
    Transforms CBA IDs to OAS post IDs.

    :param cba_ids: List of CBA identifiers
    :return: Dict of CBA IDs as keys and OAS post IDs as values
    """
    url = f"{OAS_URL}/job"
    job_ids = {}
    oas_ids = {}

    for cba_id in cba_ids:
        body = {
            "typ": "cba2oas_id",
            "subjects": [""],
            "args": {"identifier": cba_id}
        }
        res = httpx.post(url, json=body)
        job_ids[cba_id] = str(res.json())

    for cba_id, job_id in job_ids.items():
        completed = False
        job_url = url + f"/{job_id}"
        while not completed:  # this is blocking, fixme when evaluate fun works
            res = httpx.get(job_url)
            if res.json()["status"] == "completed":
                oas_ids[cba_id] = res.json()["output"]["meta"]["oas_id"]
                completed = True

    return oas_ids


def get_post(post_id: str):
    """
    Gets content of post by its ID and returns it as JSON object.

    :param post_id: OAS ID of post
    :return: Post content
    :return type: JSON
    """
    try:
        url = f"{OAS_URL}/post/{post_id}"
        res = httpx.get(url)
        res = res.json()
        return res

    except httpx.RequestError as e:
        print(f"An error occurred while requesting {e.request.url!r}.")


def get_keywords(cba_id_oas_id):
    """
    Gets keywords from devset post.

    :param cba_id_oas_id: (cba_id and oas_id) of post
    :return: Dict of CBA track ID - keywords list
    :return type: dict(cba_id: [keywords])
    """
    cba_id, oas_id = cba_id_oas_id

    post = get_post(oas_id)
    try:
        post_keywords = post["media"][0]["nlp"]["keywords"]
        return {cba_id: post_keywords}
    except Exception as e:
        print(f"Couldn't find keywords for post {post['$meta']['id']}\n{e}")


def evaluate(keywords: dict, true_keywords: dict):
    #FIXME eval, metrics param?
    pass


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="OAS NLP example scripts")
    parser.add_argument("dataset", nargs="?", default="devset/Devset.csv",
                        type=str, help="Filepath to devset CSV")
    args = parser.parse_args()

    devset_fpath = args.dataset

    """ Get true labels """
    true_labels = get_true_labels(devset_fpath)
    print(f"Ground truth: {true_labels}")  # dev print

    """ Get OAS keywords """
    cba_ids = devsetIDs_to_cbaIDs(devset_fpath)
    #print(f"CBA IDs: {cba_ids}")  # dev print
    oas_post_ids = get_post_ids(cba_ids)
    #print(f"OAS IDs: {oas_post_ids}")  # dev print
    oas_keywords = [get_keywords(ids) for ids in oas_post_ids.items()]
    print(f"Keyword List: {oas_keywords}")  # dev print

    """ Evaluate """
    # evaluate(oas_keywords, true_labels)


    # generate feed from devset
    # serve via http

    # url = 'http://localhost:6650/rss.xml'
    # res = create_feed(url)
    # print('created feed', res)
    #
    # # wait for all jobs to be finished
    # token = 'evaluate_devset'
    # while True:
    #     changes = get_changes(token)
    #     print('changes', json.dumps(changes))
        
    # get all posts
    # save posts and/or transcripts to files
    # (( optionally (or other script) to run evaluations/comparisons with previous results ))
   
    #print(f"Result:\n{res['nlp']}")
