from aiocouch import CouchDB
from aiocouch.document import Document
from aiocouch.exception import NotFoundError, ConflictError
import re
from typing import Any, Dict

from app.config import config
from app.core.util import uuid

typ_pattern = re.compile("^[a-zA-Z0-9\.]+$")

class Store:
    def __init__(self):
        couch_url = config.couch_url
        if couch_url.endswith("/"):
            couch_url = couch_url[:-1]

        self.session = CouchDB(
            config.couch_url,
            user=config.couch_user,
            password=config.couch_password
        )
        self.db_name = "oas"
        self.db = None

    async def open(self):
        self.db = await self.session.create(self.db_name, exists_ok=True)

    async def put(self, typ: str, id: str =None, value: Dict[str, Any] ={}):
        if self.db is None:
            await self.open()

        # TODO: custom exception
        # TODO: match valid type names
        if typ_pattern.search(typ) is False:
            raise Exception("Invalid type name")

        if id is None:
            id = uuid()

        couch_id = typ + "_" + id
        doc = Document(self.db, couch_id)

        try:
            await doc.fetch()
        except (NotFoundError, ConflictError) as e:
            pass

        doc["typ"] = typ
        doc["id"] = id
        doc["guid"] = couch_id
        doc["value"] = value

        await doc.save()

        return doc


    async def get(self, id: str):
        if self.db is None:
            await self.open()

        doc = await self.db.get(id)
        return doc

    async def delete(self, id: str):
        if self.db is None:
            await self.open()

        doc = Document(self.db, id)
        await doc.delete()


    async def all_by_type(self, typ: str):
        if self.db is None:
            await self.open()

        prefix = typ + "_"
        return self.db.docs(prefix=prefix)


store = Store()

