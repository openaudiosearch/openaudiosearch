from fastapi import APIRouter, Request, HTTPException
from fastapi.encoders import jsonable_encoder
from app.elastic.search import AudioObject

from app.logging import logger
from app.core.util import uuid
from app.server.jobs import jobs
from app.importer.feed import FeedManager
from app.server.models import (
    TranscriptStatus,
    TranscriptResponse,
    TranscriptRequest,
    StatusRequest,
    StatusResponse,
    JobResponse
)
from app.tasks.models import TranscribeArgs, TranscribeOpts
from app.config import config
import httpx
import json

router = APIRouter()
feed_manager = FeedManager()


@router.post("/transcript", response_model=TranscriptResponse)
def post_transcript(item: TranscriptRequest):
    args = TranscribeArgs(**item.dict())
    opts = TranscribeOpts(**item.dict())
    id = jobs.queue_job('transcribe', args, opts)
    return TranscriptResponse(id=id, status=TranscriptStatus.queued)


@router.get("/transcript/{id}", response_model=StatusResponse)
def get_status(id: str):
    result = jobs.get_result(id)
    if not result:
        return StatusResponse(id=id, status=TranscriptStatus.queued)
    # print('RESULT', result)
    return StatusResponse(id=id, status=TranscriptStatus.completed, result=result)


@router.get("/job/{id}", response_model=JobResponse)
def get_job(id: str):
    result = jobs.get_records(id)
    # print(f'RESULT {result}')
    result = JobResponse(**result)
    return result
    # if not result:
    #     return StatusResponse(id=id, status=TranscriptStatus.queued)
    # print('RESULT', result)
    # return StatusResponse(id=id, status=TranscriptStatus.completed, result=result)


@router.get("/jobs")
def get_jobs():
    list = jobs.list_jobs()
    return list


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
    body = await request.body()
    headers = {"content-type": "application/x-ndjson"}
    url = f'{config.elastic_url}{index_name}/{search_method}'
    logger.debug("Elastic-URL: " + url)
    async with httpx.AsyncClient() as client:
        r = await client.post(url, headers=headers, data=body)
        assert r.status_code == 200
        logger.debug("search result: " + r.text)
        return r.json()


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
