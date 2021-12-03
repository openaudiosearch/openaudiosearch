#!/usr/bin/env python
""" evaluate_devset

Evaluate OAS Devset:
  * Keywords
  * Transcripts (TBD)

Author(s): flipsimon, datadonk23
Date: 03.12.21
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
    :return: Dict of track ID - keywords list
    :return type: dict(ID: [keywords])
    """
    df = pd.read_csv(devset_fpath)
    return pd.Series(df.OAS_tags.str.split(",").values, index=df.ID).to_dict()


def devset_cba_ids(fpath: str):
    """
    Gets post IDs from devset spreadsheet and transforms them into OAS post
    identifiers.

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

    :param cba_identifiers: List of CBA identifiers
    :return: List of OAS post IDs
    """
    #FIXME search Elastic
    pass


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


def get_keywords(posts: list, cba_ids: list):
    """
    Gets keywords from devset posts.

    :param posts: List of devset posts.
    :param cba_ids: List of cba IDs.
    :return: Dict of CBA track ID - keywords list
    :return type: dict(ID: [keywords])
    """
    keywords = []
    for post in posts:
        try:
            keywords.append(post["nlp"]["keywords"])
        except Exception as e:
            print(f"Couldn't find keywords for post{post['$meta']['id']}\n{e}")

    return dict(zip(cba_ids, keywords))


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
    cba_ids = devset_cba_ids(devset_fpath)
    print(f"CBA IDs: {cba_ids}")  # dev print
    # oas_post_ids = get_post_ids(cba_ids)
    # devset_posts = [get_post(post_id) for post_id in oas_post_ids]
    # oas_keywords = get_keywords(devset_posts, cba_ids)

    """ Evaluate """
    # evaluate(oas_keywords, true_labels)


    # generate feed from devset
    # serve via http

    url = 'http://localhost:6650/rss.xml'
    res = create_feed(url)
    print('created feed', res)

    # wait for all jobs to be finished
    token = 'evaluate_devset'
    # while True:
    #     changes = get_changes(token)
    #     print('changes', json.dumps(changes))
        
    # get all posts
    # save posts and/or transcripts to files
    # (( optionally (or other script) to run evaluations/comparisons with previous results ))
   
    #print(f"Result:\n{res['nlp']}")
