import asyncio
from app.store.couch import store


async def async_main():
    await store.open()
    data = {
        "media_url": "http://foo.bar/baz.mp3"
    }
    typ = "AudioObject"
    doc = await store.put(typ, value=data)
    print("did put", doc)

    print("now update")
    doc = await store.get(doc.id)
    doc["value"]["media_url"] = "http://foo.bar/boo.mp3"
    udpated = await doc.save()
    print("did update!")

    all_audios = await store.all_by_type(typ)
    async for doc in all_audios:
        print("got doc", doc)


if __name__ == "__main__":
    loop = asyncio.get_event_loop()
    loop.run_until_complete(async_main())
