from vosk import Model, KaldiRecognizer, SetLogLevel
import sys
import os
import wave
import subprocess
import json
import argparse

sample_rate=16000

def transcribe(model_path, audiofile):

    process = subprocess.Popen(['ffmpeg', '-loglevel', 'quiet', '-i',
                            audiofile,
                            '-ar', str(sample_rate) , '-ac', '1', '-f', 's16le', '-'],
                            stdout=subprocess.PIPE)

    model = Model(model_path)
    rec = KaldiRecognizer(model, sample_rate)

    transcript = ""
    results = []
    while True:
        data = process.stdout.read()
        if len(data) == 0:
            break
        if rec.AcceptWaveform(data):
            recResult = json.loads(rec.Result())
            transcript = transcript + " " + recResult['text']
            results.append(recResult)
    return(results,transcript)

if __name__ == "__main__":
    # parse arguments from command line.
    parser = argparse.ArgumentParser(description="Simple Vosk example.")
    # Add an argument to define the path for the Kaldi-Model.
    parser.add_argument("-m", 
            "--model", 
            metavar="<model>", 
            type=str, 
            required=True, 
            help="The Model to use.")

    # Add an argument to define the path to the Wav Audiofile.
    parser.add_argument("-f", 
            "--file",
            metavar="<file>",
            type=str,
            required=True,
            help="The file to read the transition probabilities from."
                                                                                )
    # Parse the command line arguments.
    args = parser.parse_args()
    print(transcribe(args.model,args.file))

