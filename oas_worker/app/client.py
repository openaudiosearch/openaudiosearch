import httpx
import time
import logging

POLL_INTERVAL = 2

class JobClient(object):
    def __init__(self,
                 config,
                 logger=logging,
                 poll_interval=POLL_INTERVAL):
        self.base_url = config.base_url
        self.logger = logger
        self.poll_interval = poll_interval

    # this function blocks until a job is available.
    def poll_next_job(self, typ):
        url = f"{self.base_url}/work/{typ}"
        while True:
            res = httpx.post(url)
            if res.status_code == 200:
                res = res.json()
                return res
            elif res.status_code == 204:
                logging.debug("No work to do, waiting and polling")
                #  time.sleep(self.poll_interval)
                time.sleep(1.0)
            else:
                raise res.raise_for_status()

    def set_completed(self, job_id, patches=None, meta=None, duration=None):
        body = {
            "patches": patches,
            "meta": meta,
            "duration": duration
        }
        url = f"{self.base_url}/job/{job_id}/completed"
        res = httpx.put(url, json=body)
        res = res.json()
        return res

    def set_progress(self, job_id, progress, meta=None):
        body = {
            "progress": progress,
            "meta": meta
        }
        url = f"{self.base_url}/job/{job_id}/progress"
        res = httpx.put(url, json=body)
        res = res.json()
        return res

    def set_failed(self, job_id, error=None, meta=None, duration=None):
        body = {
            "error": error,
            "meta": meta,
            "duration": duration
        }
        url = f"{self.base_url}/job/{job_id}/failed"
        res = httpx.put(url, json=body)
        res = res.json()
        return res

    def get(self, url):
        url = f"{self.base_url}{url}"
        res = httpx.get(url)
        res = res.json()
        return res


