from vosk import Model, KaldiRecognizer, SetLogLevel
import wave
import json

model = None

def transcribe_vosk(media_id, audio_file_path, model_path):
    global model
    if not model:
        model = Model(model_path)

    rec = KaldiRecognizer(model, 16000)
    rec.SetMaxAlternatives(0)
    rec.SetWords(True)
    results = []
    parts = []
    transcript = ""

    wf = wave.open(audio_file_path, "rb")
    # print(
    #     f'WAVE INFO chan {wf.getnchannels()} sampw {wf.getsampwidth()} comptype {wf.getcomptype()}')
    if wf.getnchannels() != 1 or wf.getsampwidth() != 2 or wf.getcomptype() != "NONE":
        raise ValueError('Audio file must be WAV format mono PCM.')

    frame_chunk_len = 4000
    frames = wf.getnframes()
    count = 0
    while True:
        data = wf.readframes(frame_chunk_len)
        count += frame_chunk_len
        if len(data) == 0:
            break
        if rec.AcceptWaveform(data):
            result = json.loads(rec.Result())
            print(media_id + ' at ' + str(round((count/frames) * 100, 2)) + '%')
            #  print("RESULT", result)
            text = result['text']
            transcript = transcript + ' ' + result['text']
            if 'result' in result:
                parts = parts + result['result']
        else:
            # print(f'PARTIAL: {rec.PartialResult()}')
            rec.PartialResult()
    # TODO: Why does rec.FinalResult() not work?
    # print('FINAL')
    # print(rec.FinalResult())
    return {'text': transcript, 'parts': parts }
