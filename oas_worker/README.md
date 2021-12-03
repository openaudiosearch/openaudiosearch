# OAS Python Worker

This is a worker for Open Audio Search in Python. It is used for ASR and NLP processing.

## Running the worker locally

```
cd oas_worker
poetry install
poetry run python worker.py
```

Options:

#### `--single JOB`

Run a single job of a job type (currently `nlp` or `asr`).

## Writing jobs

A full example job can be seen in `app/examples/example.py`. To run it, a simple runner is provided in `examples`. Run it with `poetry run python examples/example-worker.py`.

---

Currently jobs reside in `app/jobs`. So first create a new file in `app/jobs`. Import `app.worker` and create a job function that uses the `worker.job` decorator to declare it as a job. Then import the file from `app/jobs/jobs.py`. From then on the job will be processed when starting the worker.

After having created the below files, start the worker:
```
poetry run python worker.py
```
Or start it with processing a single job of a single job type only and exit afterwards:

```
poetry run python worker.py --single my_job
```

Then, create a job:
```shell
curl -v -u admin:password localhost:8080/api/v1/job -X POST -d \
   '{"typ":"my_job","subjects":[""],"args":{"post_id":"somepostid"}}'
```

It should then be processed by the worker.

```python
# apps/jobs/jobs.py
import app.jobs.my_job
```

```python
# app/jobs/my_job.py
@worker.job(name="my_job")
def my_job(ctx, args):
   # ctx is a Context object (see below)
   # args is a JSON dictionary that was passed at job creation

   # ctx.get is a http client for the Core API
   post = ctx.get(args["post_id"]

   # do your processing
   new_headline = post["headline"] + " oi 🏴"

   guid = post["$meta"]["guid"]
   patch = [{
      "op": "replace",
      "path": "/headline",
      "value": new_headline
   }]
   patches = {
      guid: patch
   }
   meta = {
      "model_version": "pirate-1.0"
   }

   # return a dict with two (optional) entries:
   # { "patches": patches, "meta": meta }
   # where patches is a dict of "record guid" => json patch
   # meta can be any json and is not used apart from being saved with the job result.
   # the patches will be applied by the core.
   return {
      "patches": patches,
      "meta": meta
   }
```
