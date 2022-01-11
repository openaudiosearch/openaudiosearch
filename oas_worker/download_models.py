import os
import subprocess
import zipfile
import tarfile
import shutil
import tempfile

from app.config import config

from app.jobs.spacy_pipe import get_spacy_path, spacy_model

def download(url, path):
    p = subprocess.Popen(["curl", "--insecure", "-C", "-", "--output", path, url])
    p.wait()


def extract(filepath):
    print("extracting {filepath}")
    path, _ = os.path.split(filepath)
    with zipfile.ZipFile(filepath) as zip_ref:
        zip_ref.extractall(path)
    os.remove(filepath)
        
def download_all_models():
    download_vosk_models()
    download_diarization_models()
    download_segmentation_models()
    download_spacy_models()


def download_spacy_models():
    # spacy models are python pip packages.
    # this downloads the model to a tempdir, and then copies the path
    # to storage_path/models/spacy. this path is added to sys.path in the
    # spacy constructor at runtime.

    spacy_path = get_spacy_path()
    os.makedirs(spacy_path, exist_ok=True)
    pip_options = f'--prefix="{spacy_path}"'
    command = f'python -m spacy download {spacy_model} {pip_options}'
    os.system(command)
    #  spa
    #  ls = os.listdir(os.path.join(tempdir, 'lib'))
    #  #  path = os.path.join(tempdir, f'lib/{ls[0]}/site-packages')
    #  path = tempdir
    #  target = os.path.join(config.storage_path, 'models', 'spacy')
    #  shutil.copytree(path, target)
    #  shutil.rmtree(tempdir)


def download_vosk_models():
    models = {
        "vosk-model-spk-0.4": "https://alphacephei.com/vosk/models/vosk-model-spk-0.4.zip",
        "vosk-model-de-0.21": "https://alphacephei.com/vosk/models/vosk-model-de-0.21.zip",
        "vosk-recasepunc-de-0.21": "https://alphacephei.com/vosk/models/vosk-recasepunc-de-0.21.zip"
    }

    models_path = os.path.join(config.storage_path, "models")
    if not os.path.isdir(models_path):
        os.makedirs(models_path)

    for model in models:
        target_dir = os.path.join(models_path, model)
        target_filepath = os.path.join(models_path, model + ".zip")
        if not os.path.isdir(target_dir):
            print(f'Downloading {models[model]}')
            print(f'Downloading to {target_filepath}')
            download(models[model], target_filepath)
            os.makedirs(target_dir)
            extract(target_filepath)
        else:
            print(f'Skipping {models[model]}')


def download_segmentation_models():
    model_url = 'http://kaldi-asr.org/models/4/0004_tdnn_stats_asr_sad_1a.tar.gz'
    target_dir = os.path.join(config.storage_path, "models", "segmentation")
    if not os.path.isdir(target_dir):
        os.makedirs(target_dir)
    target_filepath = os.path.join(config.storage_path, "models", "0004_tdnn_stats_asr_sad_1a.tar.gz")
    download(model_url, target_filepath)
    with tarfile.open(target_filepath, 'r:gz') as tar_file:
        with tar_file.extractfile('exp/segmentation_1a/tdnn_stats_asr_sad_1a/final.raw') as src_file, \
            open(os.path.join(target_dir, "final.raw"), 'wb') as dest_file:
            dest_file.write(src_file.read())
        with tar_file.extractfile('exp/segmentation_1a/tdnn_stats_asr_sad_1a/post_output.vec') as src_file, \
            open(os.path.join(target_dir, "post_output.vec"), 'wb') as dest_file:
            dest_file.write(src_file.read())


def download_diarization_models():
    """Download diarization models and prepare model folder structure"""
    model_url = 'http://kaldi-asr.org/models/6/0006_callhome_diarization_v2_1a.tar.gz'
    target_dir = os.path.join(config.storage_path, "models", "diarization")
    if not os.path.isdir(target_dir):
        os.makedirs(target_dir)
    target_filepath = os.path.join(config.storage_path, "models", "0006_callhome_diarization_v2_1a.tar.gz")
    download(model_url, target_filepath)
    with tarfile.open(target_filepath, 'r:gz') as tar_file:
        with tar_file.extractfile('0006_callhome_diarization_v2_1a/exp/xvector_nnet_1a/extract.config') as src_file, \
            open(os.path.join(target_dir, "extract.config"), 'wb') as dest_file:
            dest_file.write(src_file.read())
        with tar_file.extractfile('0006_callhome_diarization_v2_1a/exp/xvector_nnet_1a/final.raw') as src_file, \
            open(os.path.join(target_dir, "final.raw"), 'wb') as dest_file:
            dest_file.write(src_file.read())
        # Prepare model for speaker embedding (xvector) extraction
        p = subprocess.Popen([
            "/kaldi/src/nnet3bin/nnet3-copy",
            f"--nnet-config={target_dir}/extract.config",
            f"{target_dir}/final.raw",
            f"{target_dir}/extract.raw"
        ], stderr=subprocess.PIPE, stdout=subprocess.PIPE)
        p.wait()
        with tar_file.extractfile('0006_callhome_diarization_v2_1a/exp/xvector_nnet_1a/xvectors_callhome1/plda') as src_file, \
            open(os.path.join(target_dir, "plda"), 'wb') as dest_file:
            dest_file.write(src_file.read())
        with tar_file.extractfile('0006_callhome_diarization_v2_1a/exp/xvector_nnet_1a/xvectors_callhome1/transform.mat') as src_file, \
            open(os.path.join(target_dir, "transform.mat"), 'wb') as dest_file:
            dest_file.write(src_file.read())
        with tar_file.extractfile('0006_callhome_diarization_v2_1a/exp/xvector_nnet_1a/xvectors_callhome1/mean.vec') as src_file, \
            open(os.path.join(target_dir, "mean.vec"), 'wb') as dest_file:
            dest_file.write(src_file.read())


if __name__ == "__main__":
    download_all_models()
