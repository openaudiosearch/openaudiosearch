from datetime import datetime
from elasticsearch_dsl import Document, Date, Integer, Keyword, Text
from elasticsearch_dsl.connections import connections

# Define a default Elasticsearch client
connections.create_connection(hosts=['localhost'])

# from elasticsearch_dsl import analyzer, tokenizer

# my_analyzer = analyzer('my_analyzer',
#     tokenizer=tokenizer('trigram', 'nGram', min_gram=3, max_gram=3),
#     filter=['lowercase']
# )

# i = Index('blog')
# i.analyzer(my_analyzer)

class AudioObject(Document):
    headline = Text(fields={'raw': Keyword()})
    identifier = Keyword()
    url = Keyword()
    contentUrl = Keyword()
    encodingFormat = Keyword()
    abstract = Text()
    description = Text(fields={'raw': Keyword()})
    creator = Text(fields={'raw': Keyword()})
    contributor= Text(fields={'raw': Keyword()})
    genre = Keyword()
    datePublished = Date()
    duration = Keyword() # TODO: change to float?
    inLanguage = Keyword()
    dateModified = Date()
    licence = Keyword()
    publisher = Text(fields={'raw': Keyword()})

    class Index:
        name = 'oas'
        settings = {
        }

# create the mappings in elasticsearch
audio = AudioObject(
    meta={'id': 42},
    abstract = 'Tobias Pfüger, MdB die Linke, berichtet aus dem "Verteidigungs"ausschuss des Bundestags am 21.April',
    contentUrl = 'https://www.freie-radios.net/mp3/20210421-abzugderbund-108544.mp3',
    contributor = ['Reinhard grenzenlos (bermuda.funk - Freies Radio Rhein-Neckar)'],
    creator = ['Reinhard grenzenlos (bermuda.funk - Freies Radio Rhein-Neckar)'],
    dateModified = 'Wed, 21 Apr 2021 16:22:58 +0200',
    datePublished = ['Wed, 21 Apr 2021 16:22:58 +0200'],
    description = 'Tobias Pfüger, MdB die Linke, berichtet aus dem "Verteidigungs"ausschuss des Bundestags am 21.April 2021',
    duration = '3:90',
    encodingFormat = 'audio/mpeg',
    genre = 'Reportage',
    headline = 'Abzug der Bundeswehr aus Afghanistan (Serie 323: Grenzenlos)',
    identifier = 'https://www.freie-radios.net/108544',
    inLanguage = ['deutsch'],
    licence = 'by-nc-sa',
    publisher = 'bermuda.funk - Freies Radio Rhein-Neckar',
    url = 'https://www.freie-radios.net/108544')
audio.save()

audio_get = AudioObject.get(id=42)
print(audio_get.headline)

# Display cluster health
print(connections.get_connection().cluster.health())