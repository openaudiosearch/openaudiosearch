#!/usr/bin/env python3

"""
OAS server bin
"""

import os
import uvicorn

# from app.server.main import app
from app.config import config
from app.elastic.search import wait_for_elastic

if __name__ == '__main__':
    wait_for_elastic()

    uvicorn.run('app.server.main:app',
                host=config.host,
                port=config.port,
                log_level=config.log_level,
                root_path=config.root_path,
                reload=config.oas_dev
                )
