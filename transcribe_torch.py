import sys
import os
import wave
import subprocess
import json
import torch
import torchaudio
from omegaconf import OmegaConf

from os import path
from pydub import AudioSegment


def preprocessAudio(samplerate, audiofile): 
    src = audiofile
    filename = audiofile[:-4]
    dst = filename + ".wav"
    sound = AudioSegment.from_mp3(src)
    sound.export(dst, format="wav")
    return dst


def init_jit_model(model_url: str,
                   device: torch.device = torch.device('cpu')):
    torch.set_grad_enabled(False)
    with tempfile.NamedTemporaryFile('wb', suffix='.model') as f:
        torch.hub.download_url_to_file(model_url,
                                       f.name,
                                       progress=True)
        model = torch.jit.load(f.name, map_location=device)
        model.eval()
    return model, Decoder(model.labels)


def transcribe_torch(audio, model_path):
    audio = preprocessAudio(16000, audio)
    print(audio)  
    device = torch.device('cpu')
    models = OmegaConf.load(os.path.join(model_path, 'models.yml'))
    print(list(models.stt_models.keys()),
      list(models.stt_models.en.keys()),
      list(models.stt_models.en.latest.keys()),
      models.stt_models.en.latest.jit)
    
    model, decoder = init_jit_model(models.stt_models.en.latest.jit, device=device)
    """
    model, decoder, utils = torch.hub.load(repo_or_dir='snakers4/silero-models',
                                           model='silero_stt',
                                           language='de',
                                           device=device)
    (read_batch, split_into_batches, read_audio, prepare_model_input) = utils  # see function signature for details

    # maybe preprocess
    batches = split_into_batches(read_audio(audio), batch_size=10)
    input = prepare_model_input(read_batch(batches[0]), device=device)

    output = model(input)

    for example in output:
        print(decoder(example.cpu()))
    """
    return NotImplemented
    