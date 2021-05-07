from celery import chain
from app.tasks.tasks import download, prepare, asr, nlp, index
from app.tasks.models import *

nlp_opts = {'pipeline': 'ner'}

res = chain(
        # download.s("https://rdl.de/sites/default/files/audio/2021/04/20210429-fokussdwest2-w23843.mp3?dl=1"),
        download.s("https://www.freie-radios.net/mp3/20210507-bibtripfolge-108890.mp3?dl=1"),
        prepare.s(16000),
        asr.s('vosk'),
        nlp.s(nlp_opts),
        index.s()
        )()

res.get()
