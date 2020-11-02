import spacy
from spacy.lang.de import German


class SpacyPipe():
    def __init__(self, pipeline):
        self.pipeline = pipeline
        self.nlp = spacy.load("de_core_news_lg")
    
    def run(self, transcript):
        """
        >>> nlp = SpacyPipe(["ner","pos"])
        >>> res = nlp.run("Samantha Bachfischer heute ist ein schöner Tag in Budapest")
        >>> print(res["ner"])
        [('Samantha Bachfischer', 'PER'), ('Budapest', 'LOC')]
        >>> print(res["pos"])
        [('Samantha', 'PROPN', 'pnc'), ('Bachfischer', 'PROPN', 'sb'), ('heute', 'ADV', 'mo'), ('ist', 'AUX', 'ROOT'), ('ein', 'DET', 'nk'), ('schöner', 'ADJ', 'nk'), ('Tag', 'NOUN', 'pd'), ('in', 'ADP', 'mnr'), ('Budapest', 'PROPN', 'nk')]
        """
        doc = self.nlp(transcript)
        ner = []
        pos = []
        missed = []
        lemma = []
        if "ner" in self.pipeline:
            for ent in doc.ents:
                ner.append((ent.text, ent.label_))
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
        return {"lemma":lemma, 
                "ner":ner, 
                "pos":pos, 
                "missed":missed}

    def create_token(self, word):
        return self.nlp.vocab.strings[word]

    