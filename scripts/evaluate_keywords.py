#!/usr/bin/env python
""" evaluate_keywords

Evaluate Keyword results from OAS on Devset.

Author(s): datadonk23
Date: 15.11.21
"""

import pandas as pd


def get_keywords():
    pass


def get_true_labels(fpath: str):
    """
    Gets ground truth labels from devset spreadsheet and returns them as a
    dict with track ID as key and keywords list as values.

    :param fpath: Filepath to devset spreadsheet
    :return: Dict of track ID - keywords list
    :return type: dict(ID: [keywords])
    """
    df = pd.read_csv(DEVSET_FPATH)
    return pd.Series(df.OAS_tags.str.split(",").values, index=df.ID).to_dict()


def evaluate(keywords: dict, true_keywords: dict):
    pass


if __name__ == '__main__':
    DEVSET_FPATH = "devset/Devset.csv"
    true_labels = get_true_labels(DEVSET_FPATH)
    oas_keywords = get_keywords()
    evaluate(oas_keywords, true_labels)
