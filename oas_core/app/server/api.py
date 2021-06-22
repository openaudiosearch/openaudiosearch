from fastapi import APIRouter, Request, HTTPException
from fastapi.encoders import jsonable_encoder
from app.elastic.search import AudioObject

from app.logging import logger
from app.core.util import uuid
from app.importer.feed import FeedManager
from app.server.jobs import Jobs

from app.server.models import (
    TranscriptStatus,
    TranscriptResponse,
    TranscriptRequest,
    StatusRequest,
    StatusResponse,
    JobResponse
)
from app.tasks.models import TranscribeArgs, TranscribeOpts
<<<<<<< HEAD
from app.config import config
import httpx
import json
from celery import chain
from app.tasks.tasks import download, prepare, asr, nlp, index

router = APIRouter()
feed_manager = FeedManager()

@router.get("/debug")
def debug():
    doc_id = "peoqCnoBuZ7gsAZn9Zof"
    audio = AudioObject.get(id=doc_id)
    print(audio.to_dict())
    audio.contentUrl = "foo"
    audio.transcript = "bar"
    print(audio.to_dict())
=======
from elasticsearch import Elasticsearch
from elasticsearch_dsl import Search

router = APIRouter()
jobs = Jobs()
>>>>>>> 97778c6 (Changed api to use celery)

@router.post("/transcript", response_model=TranscriptResponse)
def post_transcript(item: TranscriptRequest):
    # args = TranscribeArgs(**item.dict())
    # opts = TranscribeOpts(**item.dict())
    # id = jobs.queue_job('transcribe', args, opts)
    media_url = item.media_url
    result = jobs.create_transcript_job(media_url)

    return TranscriptResponse(id=result.id, status=TranscriptStatus.queued)


@router.get("/transcript/{id}", response_model=StatusResponse)
def get_status(id: str):
    result = jobs.get_job(id)
    if not result["status"] != 'SUCCESS':
        return StatusResponse(id=id, status=TranscriptStatus.queued)
    return StatusResponse(id=id, status=TranscriptStatus.completed, result=result)


@router.get("/job/{id}")
def get_job(id: str):
    result = jobs.get_job(id)
    return result


@router.get("/jobs")
def get_jobs():
    jobs_list = jobs.get_jobs()
    return jobs_list

@router.post("/feed/import")
async def import_feed(request: Request):
    body = await request.body()
    url = json.loads(body)["rss_url"]
    feed = feed_manager.get(url)
    if feed is None:
      raise HTTPException(detail="Feed exists",status_code=400)
    ids = await feed.index_and_create_tasks()
    return ids 

@router.post("/add_new_feed")
async def post_rss(request: Request):
    body = await request.body()
    url = json.loads(body)["rss_url"]
    feed = feed_manager.get(url)
    # if feed:
    #    raise HTTPException(detail="Feed exists",status_code=400)

    feed = feed_manager.put(url)
    await feed.pull()
    feed_keys = feed.get_keys()
    example_item = feed.get_example()
    schema_keys = AudioObject.get_keys()
    result = {
        "example": example_item,
        "url": url,
        "schema": schema_keys,
        "feed_keys": feed_keys,
        "mapping": None
    }

    mapping = feed_manager.get_mapping(url)
    if mapping:
        result["mapping"] = mapping
        
    return result


@router.post("/set_mapping")
async def set_mapping(request: Request):
    body = await request.body()
    body = json.loads(body)
    mapping = body["mapping"]
    url = body["rss_url"]
    logger.debug(mapping)
    feed_manager.set_mapping(url, mapping)
    mapping = feed_manager.get_mapping(url)

    return mapping


@router.post("/search/{index_name}/{search_method}")
async def search(index_name: str, search_method: str, request: Request):
    if index_name != 'oas':
        raise HTTPException(detail="Invalid index name",status_code=404)

    real_index_name = config.elastic_index

    body = await request.body()
    headers = {"content-type": "application/x-ndjson"}
    url = f'{config.elastic_url}{real_index_name}/{search_method}'
    logger.debug("Elastic-URL: " + url)
    async with httpx.AsyncClient() as client:
        r = await client.post(url, headers=headers, data=body)
        assert r.status_code == 200
        logger.debug("search result: " + r.text)
        return r.json()

<<<<<<< HEAD

#  from app.queue import queue
#  @router.post("/test-celery/", response_model=schemas.Msg, status_code=201)
#  def test_celery(
#      msg: schemas.Msg,
#      current_user: models.User = Depends(deps.get_current_active_superuser),
#  ) -> Any:
#      """
#      Test Celery worker.
#      """
#      celery_app.send_task("app.worker.main.test_celery", args=[msg.msg])
#      return {"msg": "Word received"}


#  @router.get("/search/{id}", response_model=StatusResponse)
#  def get_status(id: str):
#      return {"id": id, "status":"completed", "foo": "asdf"}
=======
@router.get("/search")
def search(query: str = ''):
    client=Elasticsearch()
    s = Search(index='audio_objects').using(client).query("match", transcript=query)
    resp = s.execute()
    return resp.to_dict()
>>>>>>> 97778c6 (Changed api to use celery)
