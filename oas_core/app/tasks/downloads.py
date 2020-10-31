import os
import wget
import zipfile

from app.config import config


def download(url, path):
    file = wget.download(url, path)
    return file


def extract(filepath):
    path, _ = os.path.split(filepath)
    with zipfile.ZipFile(filepath) as zip_ref:
        zip_ref.extractall(path)


def download_models():
    models = {
        "vosk-model-de-0.6": "https://alphacephei.com/vosk/models/vosk-model-de-0.6.zip",
        "vosk-model-spk-0.4": "https://alphacephei.com/vosk/models/vosk-model-spk-0.4.zip"
    }

    models_path = os.path.join(config.storage_path, "models")
    if not os.path.isdir(models_path):
        os.makedirs(models_path)

    for model in models:
        filepath = download(models[model], models_path)
        extract(filepath)


if __name__ == "__main__":
    download_models()
