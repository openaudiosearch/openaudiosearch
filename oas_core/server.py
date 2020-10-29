#!/usr/bin/env python3

import uvicorn
import os
from app.server.main import app

root_path = os.environ.get('ROOT_PATH') or None
print(root_path)

if __name__ == '__main__':
    uvicorn.run(app, host='0.0.0.0', port=8080, log_level='info', root_path=root_path)
