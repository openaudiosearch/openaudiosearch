#!/usr/bin/env python
""" evaluate_devset

Evaluate OAS Devset:
  * Keywords
  * Transcripts (TBD)

Author(s): flipsimon, datadonk23
Date: 05.01.22
"""

import os
import httpx
import json
import argparse
import pandas as pd
import statistics
from datetime import datetime
from string import punctuation
import spacy
from app.jobs.spacy_pipe import spacy_load


OAS_URL = 'http://admin:password@localhost:8080/api/v1'
nlp = spacy_load("de_core_news_lg")


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
    Gets ground truth labels from devset spreadsheet and lowercases them. It
    returns labels as a dict with track ID as key and keywords list as values.

    :param fpath: Filepath to devset spreadsheet
    :return: List of Dicts {cba ID: keywords list}
    :return list: list(dict(cba ID: [keywords]))
    """
    ground_truth = []
    df = pd.read_csv(devset_fpath)
    df["cba_id"] = df["URL"].apply(
        lambda s: "https://cba.fro.at/?p=" + s.split("/")[-1]
    )
    df["keywords"] = df.OAS_tags.str.lower().str.split(",").apply(
        lambda x: [s.strip() for s in x]).tolist()
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
        while not completed:  # FIXME this is blocking
            res = httpx.get(job_url)
            if res.json()["status"] == "completed":
                oas_ids[cba_id] = res.json()["output"]["meta"]["oas_id"]
                completed = True

    return oas_ids


def trigger_nlp(cba_oas_ids: dict):
    """
    Trigger NLP jobs in OAS on devset posts.
    Blocking, waits till all jobs are completed.

    :param cba_oas_ids:
    :return: Dict of CBA IDs as keys and OAS post IDs as values
    """
    url = f"{OAS_URL}/job"
    job_ids = {}

    for cba_id, oas_id in cba_oas_ids.items():
        body = {
            "typ": "nlp",
            "subjects": [""],
            "args": {"post_id": oas_id}
        }
        res = httpx.post(url, json=body)
        job_ids[cba_id] = str(res.json())

    for cba_id, job_id in job_ids.items():
        completed = False
        job_url = url + f"/{job_id}"
        while not completed:  # FIXME this is blocking
            res = httpx.get(job_url)
            if res.json()["status"] == "completed":
                completed = True


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

    :param cba_id_oas_id: (cba_id, oas_id) of post
    :return: Dict of CBA track ID - keywords list
    :return type: dict(cba_id: [keywords])
    """
    cba_id, oas_id = cba_id_oas_id

    post = get_post(oas_id)
    try:
        post_keywords = post["nlp"]["keywords"]
        return {cba_id: post_keywords}
    except Exception as e:
        print(f"Couldn't find keywords for post {post['$meta']['id']}\n{e}")


def get_transcript(cba_id_oas_id):
    """
    Gets transcript from devset post.

    :param cba_id_oas_id: (cba_id, oas_id) of post
    :return: Dict of CBA track ID - transcript
    :return type: dict(cba_id: transcript)
    """
    cba_id, oas_id = cba_id_oas_id

    post = get_post(oas_id)
    try:
        post_transcript = post["media"][0]["transcript"]["text"]
        return {cba_id: post_transcript}
    except Exception as e:
        print(f"Couldn't find transcript for post {post['$meta']['id']}\n{e}")


def flatten_oas_keywords(oas_keywords: list):
    """
    Flattens oas_keywords list to [{cba_ids: [kw]}]. Removes unused
    informations on keyword count and rank.
    Helper for evaluate_keywords().

    :param oas_keywords: List of {cba_id: (kw, count, rank)}
    :return: List of {cba_ids: [kws]}
    """
    oas_plain_keywords = []

    for keyword_dict in oas_keywords:
        cba_id = list(keyword_dict.keys())[0]
        keywords = [keyword_infos[0] for keyword_infos in
                    list(keyword_dict.values())[0]]
        oas_plain_keywords.append({cba_id: keywords})

    return oas_plain_keywords


def clean_keywords(keyword_dicts: list):
    """
    NLP keyword cleansing utility to enhance keyword comparison.
    It does:
        * Lowercasing
        * Punctuation removal
        * String stripping
        * Lemmatize keywords (resp. keyphrases)

    :param keyword_dicts: List of {cba_ids: [kws]}
    :return: cleaned keywords in list of keyword dicts
    """
    cleaned_keyword_dicts = []

    for kw_dict in keyword_dicts:
        for cba_id, kws in kw_dict.items():
            cleaned_kws = []
            for kw in kws:
                cleaned_string = (kw.lower()
                                  .translate(str.maketrans("", "", punctuation))
                                  .strip())
                cleaned_kws.append(" ".join(
                    [token.lemma_ for token in nlp(cleaned_string)]))
            cleaned_keyword_dicts.append({cba_id: cleaned_kws})

    return cleaned_keyword_dicts


def precision_recall_f1(keywords: list, true_keywords: list):
    """
    Avg precision, recall and f1 scores.

    :param keywords: List of OAS keywords {{cba_id: [kws]}}
    :param true_keywords: List of true keywords {{cba_id: [kws]}}
    :return: Average precision, average recall, average f1 score
    """
    total_prec = 0.
    total_rec = 0.

    for post_keywords in keywords:
        cba_id = list(post_keywords.keys())[0]
        oas_kws = set(post_keywords[cba_id])
        true_kws = set(next((kw_dict for kw_dict in true_keywords if cba_id
                             in kw_dict.keys()), None)[cba_id])

        if len(oas_kws) == 0 or len(true_kws) == 0:
            total_prec += 0.
            total_rec += 0.
        else:
            true_positives = len(oas_kws.intersection(true_kws))
            total_prec += true_positives / float(len(oas_kws))
            total_rec += true_positives / float(len(true_kws))

    if total_prec > 0:
        avg_prec = total_prec / float(len(true_keywords))
    else:
        avg_prec = 0.

    if total_rec > 0:
        avg_rec = total_rec / float(len(true_keywords))
    else:
        avg_rec = 0.

    if avg_prec + avg_rec > 0:
        avg_f1 = 2 * avg_prec * avg_rec / (avg_prec + avg_rec)
    else:
        avg_f1 = 0.

    return (avg_prec, avg_rec, avg_f1)


def average_precision_k(oas_kws: list, true_kws: list, k):
    """
    Average precision @ k.
    Helper for mean_average_precision_k().

    :param oas_kws: List of OAS keywords [kws]
    :param true_kws: List of true keywords [kws]
    :param k: maximum number of keywords to evaluate
    :return: average precision at k
    :return type: float
    """
    oas_kws = oas_kws[:k] if len(oas_kws) > k else oas_kws

    score = 0.
    true_positives = 0
    for i, kw in enumerate(oas_kws):
        if kw in true_kws and kw not in oas_kws[:i]:
            true_positives += 1
            score += true_positives / float(i + 1)

    return score / min(len(true_kws), k)


def mean_average_precision_k(keywords: list, true_keywords: list, k: int):
    """
    Mean average precision @ k.

    :param keywords: List of OAS keywords {{cba_id: [kws]}}
    :param true_keywords: List of true keywords {{cba_id: [kws]}}
    :param k: maximum number of keywords to evaluate
    :return: Average MAP@k
    :return type: float
    """
    if len(keywords) == 0 or len(true_keywords) == 0:
        map_at_k = 0.
    else:
        oas_kw_list = []
        true_kw_list = []
        for post_keywords in keywords:
            cba_id = list(post_keywords.keys())[0]
            oas_kw_list.append(post_keywords[cba_id])
            true_kw_list.append(next(
                (kw_dict for kw_dict in true_keywords if
                 cba_id in kw_dict.keys()), None)[cba_id])

        map_at_k = statistics.mean([
            average_precision_k(kws, true, k) for kws, true in zip(
                oas_kw_list, true_kw_list)
        ])

    return map_at_k


def evaluate_keywords(oas_keywords: list, true_keywords: list, metrics: list):
    f"""
    Evaluate OAS keywords against ground truth.
    Current available metrics: Precision, Recall, F1, MAP

    :param oas_keywords: List of {{cba_id: (kw, count, rank)}}
    :param true_keywords: List of {{cba_id: [kws]}}
    :param metrics: List of metrics to compute. available metrics [
    "Precision", "Recall", "F1", "MAP"]
    :return: Evaluation results - dict of metrics as keys and results as values
    :return type: dict
    """
    results = {}
    keywords = flatten_oas_keywords(oas_keywords)

    # Uniform NLP processing of keywords
    keywords = clean_keywords(keywords)
    true_keywords = clean_keywords(true_keywords)

    # Rankless metrics: Precision, Recall, F1
    if any(metric in metrics for metric in ["Precision", "Recall", "F1"]):
        avg_precision, avg_recall, avg_f1 = precision_recall_f1(keywords,
                                                                true_keywords)

        if "Precision" in metrics:
            results["Precision"] = f"{avg_precision:.4f}"
            print(f"Avg Prec: {avg_precision:.4f}")
        if "Recall" in metrics:
            results["Recall"] = f"{avg_recall:.4f}"
            print(f"Avg Rec: {avg_recall:.4f}")
        if "F1" in metrics:
            results["F1"] = f"{avg_f1:.4f}"
            print(f"Avg F1: {avg_f1:.4f}")

    # Rank aware metrics: MAP
    if "MAP" in metrics:
        for k in range(1, 8):
            map_at_k = mean_average_precision_k(keywords, true_keywords, k=k)
            results[f"MAP@k={k}"] = f"{map_at_k:.4f}"
            print(f"MAP@k [k={k}]: {map_at_k:.4f}")

    return results


def log_results(results: dict, oas_keywords: list, ground_truth: list,
                transcripts: list, log_path: str, duration: str):
    f"""
    Logs Evaluation results and samples data (OAS keywords, Ground truth
    keywords, OAS transcripts).

    :param results: Evaluation results - dict of metric: results
    :param oas_keywords: List of keywords - [{{cba_id: (kw, count, rank)}}]
    :param ground_truth: Ground truth keywords - [{{cba_id: [kws]}}]
    :param transcripts: List of transcripts - [{{cba_id: transcript}}]
    :param log_path: Directory to log into
    :param duration: Run time of evaluation
    :return: -
    """
    if not os.path.exists(log_path):
        os.makedirs(log_path)

    ground_truth_dict = dict([(k, v) for element in ground_truth
                              for k, v in element.items()])
    transcripts_dict = dict([(k, v) for element in transcripts
                              for k, v in element.items()])

    fpath = f"devset_evaluation_log_{datetime.now().strftime('%Y%m%d%H%M')}.txt"
    log_fpath = os.path.join(log_path, fpath)
    with open(log_fpath, "a") as f:
        f.write(f"*** KEYWORD EVALUATION RESULTS ***\n")
        for metric in results:
            f.write(f"{metric}: {results[metric]}\n")
        f.write(f"Evaluation run time: {duration}\n")
        f.write(f"\n\n*** LOG SAMPLE RESULTS ***\n")
        for oas_results in oas_keywords:
            cba_id = list(oas_results.keys())[0]
            f.write(f"\nSAMPLE {cba_id}\n")
            f.write(f"OAS KWs: {oas_results[cba_id]}\n")
            f.write(f"Ground Truth KWs: {ground_truth_dict[cba_id]}\n")
            f.write(f"Transcipt: {transcripts_dict[cba_id]}\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="OAS Keyword example script")
    parser.add_argument("dataset", nargs="?",
                        default="devset/assets/Devset.csv",
                        type=str, help="Filepath to devset CSV")
    parser.add_argument("log_path", nargs="?",
                        default="devset/evaluations",
                        type=str, help="Path to log evaluation")
    args = parser.parse_args()
    devset_fpath = args.dataset
    log_path = args.log_path
    start = datetime.now()  # track run time of evaluation

    """ Get true labels """
    true_labels = get_true_labels(devset_fpath)

    """ Get OAS keywords & transcript """
    cba_ids = devsetIDs_to_cbaIDs(devset_fpath)
    oas_post_ids = get_post_ids(cba_ids)
    trigger_nlp(oas_post_ids)
    oas_keywords = [get_keywords(ids) for ids in oas_post_ids.items()]
    oas_transcripts = [get_transcript(ids) for ids in oas_post_ids.items()]

    """ Evaluate """
    keyword_metrics = ["Precision", "Recall", "F1", "MAP"]
    eval_results = evaluate_keywords(oas_keywords, true_labels, keyword_metrics)

    """ LOG Results """
    end = datetime.now()
    eval_duration = str(end - start)
    log_results(eval_results, oas_keywords, true_labels, oas_transcripts,
                log_path, eval_duration)


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
