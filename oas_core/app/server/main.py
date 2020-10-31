from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles
from starlette.middleware.cors import CORSMiddleware
from starlette.responses import RedirectResponse
import os

from app.server.api import router as api_router
from app.config import config

api_v1_prefix = '/oas/v1'

app = FastAPI(
    title="Open Audio Search API",
    version="1.0",
    description="Open Audio Search API server",
    openapi_url=f"{api_v1_prefix}/openapi.json"
)


@app.get("/", include_in_schema=False)
def docs_redirect():
    ui_path = config.root_path + '/ui'
    return RedirectResponse(ui_path)


static_path = os.path.abspath('../frontend/dist')
app.mount("/ui", StaticFiles(directory=static_path, html=True), name="static")

# Set all CORS enabled origins
#  if settings.BACKEND_CORS_ORIGINS:
app.add_middleware(
    CORSMiddleware,
    allow_origins=['*'],
    # allow_origins=[str(origin)
    #                for origin in settings.BACKEND_CORS_ORIGINS],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


app.include_router(api_router, prefix=api_v1_prefix)
