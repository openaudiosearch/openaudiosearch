# Development setup

Follow the instructions for the development setup in the [README](../README.md).

# Tips and tricks

## Development mode

The server can be reloaded automatically when application code changes. You can enable it by setting the `oas_dev` env config, or starting the server with `OAS_DEV=1 server.py`.

## Frontend development

### Requirements

You need Node.js and npm or yarn. yarn is recommended because it's much faster.

On Debian based systems use the following to install both Node.js and yarn:
```bash
curl -sL https://deb.nodesource.com/setup_12.x | sudo -E bash -
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
sudo apt update
sudo apt install yarn nodejs
```

#### Development

For development `webpack-dev-server` is included. In this folder, run `yarn` to install all dependencies and then `yarn start` to start the live-reloading development server. Then open the UI at [http://localhost:4000](http://localhost:4000). In development mode, the UI expects a running oas_worker server at `http://localhost:8080`.

#### Deployment

Make sure to run `yarn build` in this directory after pulling in changes. The `oas_worker` server serves the UI at `/ui` from the `dist/` folder in this directory. 


## Inspect the Redis databaes

[`redis-commander`](https://www.npmjs.com/package/redis-commander) is a useful tool to inspect the Redis database. 

```bash
# install redis-commander
yarn global add redis-commander
# or: npm install -g redis-commander

# start redis-commander
redis-commander
```

Now, open your browser at [http://localhost:8081/](http://localhost:8081/)


## ASR Evaluation

Start worker:
```
cd oas_worker
python worker.py
```

Run transcription using ASR engine in another Terminal:
```
cd oas_worker
# download models if needed
python task-run.py download_models
# transcribe a single file
python task-run.py asr --engine ENGINE [--language LANGUAGE] --file_path FILE_PATH [--help]
# (e.g). 
python task-run.py asr --engine vosk --file_path ../examples/frn-leipzig.wav
```

## NLP Evaluation

### Generate Devset

Generate and serve Devset RSS feed on localhost port 6650:  
```
cd scripts/
python generate_devset.py
sh serve_nlp_devset.sh
```

Import RSS feed:  
In UI, login first. Then head over to `Importer`-Tab. There fill in the URL 
`http://127.0.0.1:6650/rss.xml` into the `Add new feed`-Section and make sure 
to check the `Transcribe items`-Button.


### Access NLP Results

In Search-UI, click on a post you want to inspect. From its URL, copy the 
Post-ID and paste it as argument to the examples nlp.py script:
```
cd oas_worker/examples
poetry run python nlp.py <OAS-POST-ID>
```
