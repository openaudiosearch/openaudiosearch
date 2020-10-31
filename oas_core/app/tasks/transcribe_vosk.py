from vosk import Model, KaldiRecognizer, SetLogLevel
import wave
import json


def transcribe_vosk(audio, model_path):
    model = Model(model_path)
    rec = KaldiRecognizer(model, 16000)
    results = []
    wave_frames = wave.open(audio, "rb")
    transcript = ""
    while True:
        data = wave_frames.readframes(4000)
        if len(data) == 0:
            break
        if rec.AcceptWaveform(data):
            recResult = json.loads(rec.Result())
            transcript = transcript + " " + recResult['text']
            results.append(recResult)
        else:
            rec.PartialResult()
    # TODO: Why does rec.FinalResult() not work?
    return results