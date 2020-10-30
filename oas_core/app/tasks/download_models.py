import os
import wget
import zipfile

from app.config import config


models = {
    "vosk-model-de-0.6": "https://alphacephei.com/vosk/models/vosk-model-de-0.6.zip",
    "vosk-model-spk-0.4": "https://alphacephei.com/vosk/models/vosk-model-spk-0.4.zip"
    #FIXME de-deepspeech-0.74 https://drive.google.com/drive/folders/1PFSIdmi4Ge8EB75cYh2nfYOXlCIgiMEL
}


def download(url, path):
    wget.download(url, path)


def extract(path, model):
    with zipfile.ZipFile(os.path.join(path, model + ".zip"), "r") as zip_ref:
        zip_ref.extractall(path)


models_path = os.path.join(config.storage_path, "models")
if not os.path.isdir(models_path):
    os.makedirs(models_path)

for model in models:
    download(models[model], models_path)
    extract(models_path, model)
