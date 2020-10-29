from fastapi import FastAPI
from starlette.middleware.cors import CORSMiddleware
from starlette.responses import RedirectResponse

from app.server.api import router as api_router
#  from app.core.config import settings


api_v1_prefix = '/oas/v1'

app = FastAPI(
    title="Open Audio Search API",
    version="1.0",
    description="Open Audio Search API server",
    openapi_url=f"{api_v1_prefix}/openapi.json"
)

@app.get("/", include_in_schema=False)
def docs_redirect():
    return RedirectResponse(f"/docs")

# Set all CORS enabled origins
#  if settings.BACKEND_CORS_ORIGINS:
#      app.add_middleware(
#          CORSMiddleware,
#          allow_origins=[str(origin) for origin in settings.BACKEND_CORS_ORIGINS],
#          allow_credentials=True,
#          allow_methods=["*"],
#          allow_headers=["*"],
#      )

app.include_router(api_router, prefix=api_v1_prefix)
