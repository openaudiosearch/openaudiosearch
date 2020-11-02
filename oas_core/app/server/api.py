from fastapi import APIRouter
from fastapi.encoders import jsonable_encoder

from app.core.util import uuid
from app.server.jobs import jobs
from app.server.models import (
    TranscriptStatus,
    TranscriptResponse,
    TranscriptRequest,
    StatusRequest,
    StatusResponse,
    JobResponse
)
from app.tasks.models import TranscribeArgs, TranscribeOpts

from app.elastic.search import SearchIndex

router = APIRouter()


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


# @router.get("/search")
# def search(query):
#     response = SearchIndex.search(query)
#     return response

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
