import feedparser
import json


class RSSImport:
    def __init__(self, url):
        self.url = url
        self.mapping = None
        self.feed = None
        self.keys = None
        self.items = None

    def pullFeed(self):
        self.feed = feedparser.parse(self.url)
        self.keys = list(self.feed.entries[0].keys())
        self.items = self.feed.entries

    def getKeys(self):
        if self.keys is not None:
            return self.keys
        else:
            raise Exception("no keys")

    def mapFields(self, mapping):
        for entry in self.feed.entries:
            {
                "headline": entry[json.loads(mapping)["headline"]],
            }
            print()
