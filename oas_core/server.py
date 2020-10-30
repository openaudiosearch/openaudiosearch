#!/usr/bin/env python3

"""
OAS server bin
"""

import os
import uvicorn

from app.server.main import app
from app.config import config


if __name__ == '__main__':
    uvicorn.run(app, host=config.host, port=config.port,
                log_level=config.log_level, root_path=config.root_path)
