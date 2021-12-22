#!/usr/bin/env python
""" transcript_word_frequencies

Word frequencies in transcripts of Devset.
#TODO automate transcript download in case devset grows in size

Author(s): datadonk23
Date: 11.11.21
"""

import glob
import collections
import nltk
from nltk.corpus import stopwords
from nltk.stem.cistem import Cistem


def get_txt(transcript_fpath: str):
    """
    Read in transcript.

    :param transcript_fpath: Filepath of transcript
    :return: Text string of transcript
    """

    with open(transcript_fpath, "r") as file:
        return file.read()


def clean_txt(transcript_text: str):
    """

    :param transcript_text: Raw transcript text
    :return: Cleaned transcript text
    """

    cleaned_tokens = []
    stemmer = Cistem(case_insensitive=False)
    custom_stopwords = {"ja", "nein", "nicht"}
    stop_words = set(stopwords.words("german")).union(custom_stopwords)

    for word in transcript_text.split():
        if word not in stop_words:
            stem_token = stemmer.stem(word)
            cleaned_tokens.append(stem_token)

    return " ".join(cleaned_tokens)


def word_frequencies(transcript_text: str, counter: collections.Counter):
    """
    Calculates word frequencies in given transcript.

    :param transcript_text: Text string of transcript
    :param counter: Counter instance
    :return: Word frequencies
    :return type: collections.Counter
    """

    counter.clear()
    counter.update(clean_txt(transcript_text).split())

    return counter.most_common()


def top_n_words(word_frequencies: list, n: int):
    """
    Top n words and occurrence in word frequency list.

    :param word_frequencies: List of (word, occurrence) tuples
    :return: top n (word, occurrence) tuples
    :return type: list
    """

    return word_frequencies[:n]


if __name__ == '__main__':
    """ Prints Top-n words in Devset transcripts. """
    TOP_N = 15
    SOURCES_DIR = "../../examples/Devset"
    transcript_fpaths = [fpath for fpath in glob.glob(SOURCES_DIR +
                                                      "/*_transcript.txt")]

    for transcript_fpath in transcript_fpaths:
        id = transcript_fpath.split("/")[-1].split("_")[0]
        counter = collections.Counter()
        text = get_txt(transcript_fpath)
        word_frequency_list = word_frequencies(text, counter)
        print(f"Transcript {id}:\n{top_n_words(word_frequency_list, TOP_N)}\n")
