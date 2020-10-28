import click

from transcribe_vosk import transcribe_vosk
from transcribe_torch import transcribe_torch
import sys
import os
import wave
import subprocess
import json

@click.group()
def cli():
    pass

@cli.command()
@click.option('--engine','-e',
              type=click.Choice(['vosk', 'deepspeech', 'torch'], case_sensitive=False), default='vosk', help="default: vosk")
#@click.option('--nlp', '-n', type=click.Choice(['spacy', 'stanza'], case_sensitive=False))
#@click.option('--nlppipeline', '-p', type=click.Choice(['ner', 'pos', 'dep', 'ner + pos'], case_sensitive=False))
@click.argument('audiofile', type=str)
@click.argument('model_path')
def transcribe(audiofile, engine, model_path):
    result = []
    print(engine)
    if engine == "vosk":
        transcript = transcribe_vosk(audiofile, model_path)
    elif engine == "deepspeech":
        transcript = transcribe_deepspeech(audiofile, model_path)
    elif engine == "torch":
        transcript = transcribe_torch(audiofile, model_path)
    click.echo(transcript)
    return transcript    

@cli.command()
def status():
    click.echo('Statusmessage')

if __name__ == '__main__':
    cli()
