from typing import Dict, List, Optional
from pydantic import BaseModel, Schema
from enum import Enum


class TranscriptStatus(str, Enum):
    queued = "queued"
    processing = "processing"
    completed = "completed"


class TranscriptRequest(BaseModel):
    media_url: str
    language: str = "en"
    asr_engine: str = "vosk"
    nlp_engine: str = "spacy"
    nlp_pipeline: str = "ner"


class TranscriptResponse(BaseModel):
    id: str
    status: TranscriptStatus


class StatusRequest (BaseModel):
    id: str


class StatusResponse(TranscriptResponse):
    foo: str
