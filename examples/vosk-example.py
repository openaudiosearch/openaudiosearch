from vosk import Model, KaldiRecognizer, SetLogLevel
import sys
import os
import wave
import subprocess
import json
import argparse
import spacy
import deepspeech

sample_rate = 16000


def preprocessAudio(samplerate, audiofile):
    process = subprocess.Popen(['ffmpeg', '-loglevel', 'quiet', '-i',
                            audiofile,
                            '-ar', str(sample_rate), '-ac', '1', '-f', 's16le', '-'],
                            stdout=subprocess.PIPE)
    return process


def transcribe(model_path, process, asr_engine="vosk"):
    if asr_engine == "vosk":
        results, transcript = transcribe_vosk(model_path, process)
    elif asr_engine == "deepspeech":
        results, transcript = transcribe_deepspeech(model_path, process)
    return (results, transcript)


def transcribe_vosk(model_path, process):
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
    return (results, transcript)


def transcribe_deepspeech(model_path, process):
    model = deepspeech.Model(model_path)


def nlp(transcript, engine, pipe):
    if engine[0] == "spacy":
        result = spacy_nlp(engine[1], transcript, pipe)
    elif engine[0] == "stanza":
        result = stanza_nlp(engine[1], transcript, pipe)
    return result


def spacy_nlp(nlp, transcript, pipe):
    doc = nlp(transcript)
    res = []
    if "ner" in pipe:
        for ent in doc.ents:
            res.append((ent.text, ent.label_))
    if "pos" in pipe:
        for token in doc:
            res.append((token.text, token.pos_, token.dep_))
    return res

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
            help="Path to Audiofile")
    
    # Add an argument to define the path for the Kaldi-Model.
    parser.add_argument("-n", 
            "--nlp", 
            metavar="<nlp>", 
            type=str, 
            required=True, 
            help="The NLP Engine to use.")
    parser.add_argument("-p",
            '--pipe',
            metavar="<pipeline>", 
            nargs='+',
            help='list of nlp tasks (e.g "ner" "pos")'
                                                                                )
    # Parse the command line arguments.
    args = parser.parse_args()
    if args.nlp == "spacy":
        nlp = ("spacy", spacy.load("de_core_news_md"))
    process= preprocessAudio(16000, args.file)
    results, transcript = transcribe(args.model, process, "deepspeech")
    print(transcript)
    #res = nlp(transcript, nlp, args.pipeline)
    #print(res)

