import httpx
import asyncio
import feedparser
from app.logging import logger
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

    async def pull(self):
        async with httpx.AsyncClient() as client:
            try:
                response = await client.get(self.url)
                raw_feed = response.text
                self.feed = feedparser.parse(raw_feed)
                if not self.feed.get("entries"):
                    raise Exception(f"URL {self.url} can not be parsed as feed or feed is empty.")
                self.keys = list(self.feed.entries[0].keys())
                self.items = self.feed.entries
                return True
            except httpx.RequestError as exc:
                raise Exception(f"An error occurred while requesting {exc.request.url!r}.")
            except httpx.HTTPStatusError as exc:
                raise Exception(f"Error response {exc.response.status_code} while requesting {exc.request.url!r}.")
            except Exception as exc:
                raise exc


    def get_example(self):
        return self.items[0]

    def get_keys(self):
        if self.keys is not None:
            return self.keys
        else:
            self.keys = set()
            for item in self.items:
                item_keys = item.keys
                for key in item_keys:
                    if key not in self.keys:
                        self.keys.add(key)
            return self.keys

    def map_fields(self, mapping):
        for entry in self.feed.entries:
            {
                "headline": entry[json.loads(mapping)["headline"]],
            }
            print()
