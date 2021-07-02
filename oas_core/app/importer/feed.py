import httpx
import asyncio
import feedparser
import json

from app.logging import logger
from app.elastic.search import AudioObject
from app.server.jobs import jobs
from app.tasks.models import TranscribeArgs, TranscribeOpts


class FeedManager:

    def __init__(self):
        # TODO load store from db
        self.store = {}
        self.mappings = {}

    def put(self, feed_url):
        feed = self.store.get(feed_url)
        if not feed:
            mapping = self.get_mapping(feed_url)
            feed = Feed(feed_url, mapping=mapping)
            self.store[feed_url] = feed
        return feed

    def get(self, feed_url):
        return self.store.get(feed_url)

    def set_mapping(self, feed_url, mapping):
        # TODO: Check mapping
        self.mappings[feed_url] = mapping
        if self.store.get(feed_url) is not None:
            self.store[feed_url].mapping = mapping

    def get_mapping(self, feed_url):
        mapping = self.mappings.get(feed_url)
        if mapping is None:
            return {
                "headline": "title",
                "identifier": "id",
                "url": "link",
                "abstract": "subtitle",
                "description": "summary",
                "creator": "author",
                "datePublished": "published",
                "publisher": "frn_radio", # TODO: Remove from default
                "genre": "frn_art", # TODO: Remove from default
            }
        return mapping


class Feed:
    def __init__(self, url, mapping=None):
        self.url = url
        self.mapping = mapping
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
                    raise Exception(
                        f"URL {self.url} can not be parsed as feed or feed is empty.")
                self.items = self.feed.entries
                return True
            except httpx.RequestError as exc:
                raise Exception(
                    f"An error occurred while requesting {exc.request.url!r}.")
            except httpx.HTTPStatusError as exc:
                raise Exception(
                    f"Error response {exc.response.status_code} while requesting {exc.request.url!r}.")
            except Exception as exc:
                raise exc

    async def index_and_create_tasks(self):
        if self.feed is None:
            await self.pull()

        audio_objects = self.transform()
        ids = []
        # put into index
        for audio_object in audio_objects:
            result = audio_object.save()
            print("saved doc", result, audio_object.meta.id)
            doc_id = audio_object.meta.id
            #  args = TranscribeArgs(media_url=audio_object.contentUrl, doc_id=doc_id)
            #  opts = TranscribeOpts(engine='vosk')
            job_id = jobs.create_transcript_job(audio_object.contentUrl, doc_id)
            print("created job:", job_id)
            ids.append({ "job": str(job_id), "doc": doc_id })

        return ids
            

    def get_example(self):
        return self.items[0]

    def get_keys(self):
        if self.keys is not None:
            return self.keys
        else:
            self.keys = set()
            for item in self.feed.entries:
                item_keys = item.keys()
                for key in item_keys:
                    if key not in self.keys:
                        self.keys.add(key)
            return self.keys

    def transform(self):
        if self.mapping is None:
            raise Exception("Mapping is missing")
        docs = []
        mapping = self.mapping
        logger.debug(mapping)
        for entry in self.feed.entries:
            doc = AudioObject()
            for key in mapping.keys():
                setattr(doc, key, entry.get(mapping[key]))
            if entry.enclosures and entry.enclosures[0]:
                doc.contentUrl = entry.enclosures[0].href
                doc.encodingFormat = entry.enclosures[0].type
            print("TRANSFORMED", doc)
            docs.append(doc)
        return docs
