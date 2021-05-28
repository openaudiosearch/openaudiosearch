import feedparser
import json


class FeedManager:

    def __init__(self):
        # TODO load store from db
        self.store = {}
        self.mappings = {}

    def put(self, feed_url):
        feed = self.store.get(feed_url)
        if not feed:
            feed = RSSImport(feed_url)
            self.store[feed_url] = feed
        return feed

    def get(self, feed_url):
        return self.store.get(feed_url)

    def set_mapping(self, feed_url, mapping):
        # TODO: Check mapping
        self.mappings[feed_url] = mapping



class RSSImport:
    def __init__(self, url):
        self.url = url
        self.mapping = None
        self.feed = None
        self.keys = None
        self.items = None

    # This should be async
    def pull(self):
        # check url is rss feed
        self.feed = feedparser.parse(self.url)
        self.keys = list(self.feed.entries[0].keys())
        self.items = self.feed.entries

    def get_keys(self):
        if self.keys is not None:
            return self.keys
        else:
            # here should no error happen
            raise Exception("no keys")

    def map_fields(self, mapping):
        for entry in self.feed.entries:
            {
                "headline": entry[json.loads(mapping)["headline"]],
            }
            print()
