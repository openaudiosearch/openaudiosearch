from fastapi import APIRouter
from fastapi.encoders import jsonable_encoder

from app.server.models import (
    TranscriptResponse,
    TranscriptRequest,
    StatusRequest,
    StatusResponse,
)

router = APIRouter()

@router.post("/transcript", response_model=TranscriptResponse)
def post_transcript(item: TranscriptRequest):
    return {"id": task.id, "status":"queued"}

@router.get("/transcript/{id}", response_model=StatusResponse)
def get_status(id: str):
    return {"id": id, "status":"completed", "foo": "asdf"}



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
