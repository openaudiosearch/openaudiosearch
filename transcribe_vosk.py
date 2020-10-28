from vosk import Model, KaldiRecognizer, SetLogLevel
import sys
import os
import wave
import subprocess
import json
def preprocessAudio(samplerate, audiofile):
    process = subprocess.Popen(['ffmpeg', '-loglevel', 'quiet', '-i',
                            audiofile,
                            '-ar', str(samplerate), '-ac', '1', '-f', 's16le', '-'],
                            stdout=subprocess.PIPE)
    return process


def transcribe_vosk(audio, model_path):
    model = Model(model_path)
    rec = KaldiRecognizer(model, 16000)

    transcript = ""
    results = []
    process = preprocessAudio(16000,audio)
    while True:
        data = process.stdout.read()
        if len(data) == 0:
            break
        if rec.AcceptWaveform(data):
            recResult = json.loads(rec.Result())
            transcript = transcript + " " + recResult['text']
            results.append(recResult)
    return (results, transcript)


