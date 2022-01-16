#!/usr/bin/env python
""" evaluate_devset_utils

Utility functions for keyword evaluation on devset:
    * Metric Calculation
    * Logging

Author(s): datadonk23
Date: 05.01.22
"""

import os
import statistics
from datetime import datetime


### Metric Calculation Utils

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


### Logging Utils

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
