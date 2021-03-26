from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles
from starlette.middleware.cors import CORSMiddleware
from starlette.responses import RedirectResponse, HTMLResponse
import os

from app.server.api import router as api_router
from app.config import config

api_v1_prefix = '/oas/v1'
static_path = config.frontend_path

app = FastAPI(
    title="Open Audio Search API",
    version="1.0",
    description="Open Audio Search API server",
    openapi_url=f"{api_v1_prefix}/openapi.json"
)


def read_index_html():
    path = os.path.join(static_path, 'index.html')
    # Open a file: file
    with open(path, mode='r') as file:
        # read all lines at once
        text = file.read()
        script = f'<script>window.OAS_ROOT_PATH="{config.root_path}";</script>'
        text = text.replace('</head>', script + '</head>')
        return text


index_html = read_index_html()


@app.get("/", include_in_schema=False)
def docs_redirect():
    ui_path = config.root_path + '/ui/index.html'
    return RedirectResponse(ui_path)


@app.get("/ui/index.html", include_in_schema=False)
def get_index_html():
    response = HTMLResponse(index_html)
    return response


app.mount("/ui", StaticFiles(directory=static_path,
                             html=True), name="static")


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
