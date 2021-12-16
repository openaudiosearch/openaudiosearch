import spacy
import pytextrank
import os
import sys
from spacy.lang.de import German
from pathlib import Path

from app.config import config
from app.logging import logger

spacy_model = "de_core_news_lg"

def get_spacy_path() -> Path:
    return os.path.join(config.storage_path, 'models', 'spacy')

def ensure_spacy_import_path() -> bool:
    path = get_spacy_path()
    try:
        ls = os.listdir(os.path.join(path, 'lib'))
        if len(ls) < 1:
            return False

        py_version = ls[0]
        module_path = os.path.join(path, 'lib', py_version, 'site-packages')
        if not module_path in sys.path:
            sys.path.append(module_path)
        return True
    except Exception as err:
        return False

def spacy_load(model):
    model_path_exists = ensure_spacy_import_path()
    if model_path_exists:
        try:
            nlp = spacy.load(model)
            logger.info(f"loaded spaCy with model `{model}`")
            return nlp
        except Exception as err:
            logger.warning(f"disabling spaCy: missing model `{model}`")
            #  logger.exception(err)
            logger.warning("run `python download_models.py` to download the models.")
            return None
    else:
        logger.warning("disabling spaCy: missing models")
        logger.warning("run `python download_models.py` to download the models.")
        return None

class SpacyPipe():
    def __init__(self, pipeline):
        self.pipeline = pipeline
        # Will be None if the model cannot be loaded!
        self.nlp = spacy_load(spacy_model)
        self.nlp.add_pipe("textrank")
    
    def run(self, transcript):
        """
        >>> nlp = SpacyPipe(["ner","pos"])
        >>> res = nlp.run("Samantha Bachfischer heute ist ein schöner Tag in Budapest")
        >>> print(res["ner"])
        [('Samantha Bachfischer', 'PER'), ('Budapest', 'LOC')]
        >>> print(res["pos"])
        [('Samantha', 'PROPN', 'pnc'), ('Bachfischer', 'PROPN', 'sb'), ('heute', 'ADV', 'mo'), ('ist', 'AUX', 'ROOT'), ('ein', 'DET', 'nk'), ('schöner', 'ADJ', 'nk'), ('Tag', 'NOUN', 'pd'), ('in', 'ADP', 'mnr'), ('Budapest', 'PROPN', 'nk')]
        """
        if self.nlp is None:
            return {}

        doc = self.nlp(transcript)
        ner = []
        pos = []
        missed = []
        lemma = []
        keywords = []
        if "ner" in self.pipeline:
            for ent in doc.ents:
                ner.append((ent.text, ent.label_, ent.start, ent.end))
        token_based = ("pos" or "missed" or "lemma") in self.pipeline
        if token_based:   
            for token in doc:
                if "pos" in self.pipeline:
                    pos.append((token.text, token.pos_, token.dep_))
                if "missed" in self.pipeline:
                    _token = self.nlp.vocab.strings[token.text]
                    if _token not in self.nlp.vokab:
                        missed.append(token.text)
                if "lemma" in self.pipeline:
                    lemma.append(token.text, token.lemma)

        if "textrank" in self.pipeline:
            max_n = 10
            if len(doc._.phrases) < max_n:
                best_n = len(doc._.phrases)
            else:
                best_n = max_n

            for phrase in doc._.phrases[:best_n]:
                keyword = (phrase.text, phrase.count, phrase.rank)
                keywords.append(keyword)

        return {"lemma":lemma, 
                "ner":ner, 
                "pos":pos, 
                "missed":missed,
                "keywords":keywords}

    def create_token(self, word):
        return self.nlp.vocab.strings[word]

    
