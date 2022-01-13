# Devset

Devset is our own development dataset to evaluate transcript quality and keyword extraction performance. It actually consists of 20 curated and manually labeled samples from [cba](https://cba.fro.at/) and should grow in size in the near future. You can find the dataset as `Devset.csv` (as well as a RSS feed generated consisting of its samples: `rss.xml`) in `oas_worker/devset/assets` directory.


## Evaluation

You can do an evaluation run either on our OAS devset or generate your own devset feed (which is described in the below section). Follow these steps to run evaluation on the OAS devset:  
1. Optional: Generate a devset (see below). A demo devset is included in the repo.
2. Start the required services via Docker (see the [docs on the Docker test setup](../../docker/test/README.md))
3. Start OAS core (`cargo run -- run`) and a worker (`cd oas_worker; poetry run python worker.py`). Detail are in the "Development and local setup" section in the [README](../../README.md).
4. Start the evaluation script. The script can optionally serve the devset and add it to OAS, which - when started from a clean and empty environment - will perform all required steps to get an evaluation.

```
poetry run python evaluate_devset.py [DATASET] [LOG_PATH] [--asr] [--feed]
```

To run the evaluation script on the demo devset, while serving the devset feed and also running the ASR jobs, use the following command:

```
cd oas_worker
poetry run python evaluate_devset.py --asr --feed
```

By using the optional arguments you can specify the filepath to the devset (\[DATASET\]) and path in which logging results should be persisted (\[LOG_PATH]\). E.g., `poetry run python evaluate_devset.py devset/assets/Devset.csv devset/evaluation`, which are the default arguments used by OAS.

Per default evaluation is done by using all currently available metrics, which are Precsion, Recall, F1 and MAP@k at the moment.


## Generate and serve devset

If you want to generate your own devset feed:

1. Change samples data in Devset spreadsheet (`oas_worker/devset/assets/Devset.csv`).
2. To generate the custom RSS Feed, run in `oas_worker/` directory:
  ```
  poetry run python devset/generate_devset.py
  ```
3. Proceed with an evaluation run, as described in the section above.


