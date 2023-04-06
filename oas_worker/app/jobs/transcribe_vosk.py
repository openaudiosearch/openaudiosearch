from vosk import Model, KaldiRecognizer, SetLogLevel
import os
from pydub import AudioSegment
import time
import numpy as np
from pydiar.models import BinaryKeyDiarizationModel
from pydiar.util.misc import optimize_segments
from pprint import pprint
import json

from app.jobs.asr.voice import get_voice_segments
from app.jobs.recasepunc.recasepunc import perform_recase

SAMPLE_RATE = 16000

vosk_model = None
diar_model = None


def load_audio(audio_file_path):
    audio = AudioSegment.from_wav(audio_file_path)
    audio = audio.set_frame_rate(SAMPLE_RATE)
    audio = audio.set_channels(1)
    return audio

def diarization_segments(audio):
    global diar_model
    if not diar_model:
        diar_model = BinaryKeyDiarizationModel()
    segments = diar_model.diarize(
        SAMPLE_RATE, np.array(audio.get_array_of_samples())
    )
    optimized_segments = optimize_segments(segments)
    def transpose_segment(seg):
        return {"start": seg.start,"end":seg.start + seg.length, "type": "speaker", "data": int(seg.speaker_id) }
    tranposed_segments = map(transpose_segment, optimized_segments)
    return tranposed_segments


def perform_transcription(model_path, audio, sample_rate=SAMPLE_RATE, on_progress=None):
    global vosk_model
    if not vosk_model:
        ctx.log.trace(f"loading vosk model from {model_path}")
        vosk_model = Model(model_path)
        ctx.log.trace(f"model loaded")
    rec = KaldiRecognizer(vosk_model, sample_rate)
    rec.SetMaxAlternatives(0)
    rec.SetWords(True)
    ctx.log.trace(f"vosk kaldi recognizer initialized")
    chunk_ms = 2000 # chunk size in miliseconds
    duration = len(audio) # total audio duration in miliseconds

    #  timer = time.time()
    start = 0
    end = 0
    while end < duration:
        start = end
        end = min(start + chunk_ms, duration)

        # perform ASR
        data = audio[start : end] # audio samples in miliseconds
        rec.AcceptWaveform(data.get_array_of_samples().tobytes())

        # report progress
        progress = end / duration
        #  time_remaining = round((time.time() - timer) * (1 / progress), 1)
        if on_progress:
            on_progress(progress)
        

    vosk_result = json.loads(rec.FinalResult())
    return vosk_result

def transcribe_vosk(ctx, media_id, audio_file_path, voice_activity=True, diarization=True, recasepunc=True):
    segments = []
    meta = {}

    # get voice activity
    if voice_activity:
        timer = time.time()
        ctx.log.trace("starting voice activity detection")
        voice_segments = get_voice_segments(audio_file_path, sample_rate=SAMPLE_RATE)
        segments.extend(voice_segments)
        # TODO: model version?
        meta["vad"] = { "engine": "silero-vad", "time": round(time.time() - timer, 2)}
        ctx.log.trace("finished voice activity detection")

    # load audio
    audio = load_audio(audio_file_path)

    # perform diarization
    if diarization:
        timer = time.time()
        ctx.log.trace("starting speaker diarization")
        diar_segments = diarization_segments(audio)
        # TODO: model version?
        meta["diar"] = {"engine": "pydiar", "time": round(time.time() - timer, 2)}
        segments.extend(diar_segments)
        ctx.log.trace("finished speaker diarization")
    
    # perform transcription
    ctx.log.trace("starting vosk transcription")
    timer = time.time()
    model_base_path = ctx.config.model_path
    vosk_model = ctx.config.model
    vosk_model_path = os.path.join(model_base_path, vosk_model)
    def on_progress(progress):
        ctx.log.trace(f"  transcribing {media_id} - progress {str(round(progress*100, 1))}%")
        ctx.set_progress(progress)

    vosk_result = perform_transcription(ctx, vosk_model_path, audio, sample_rate=SAMPLE_RATE, on_progress=on_progress)
    meta["asr"] = {"engine":"vosk","model":vosk_model,"time": round(time.time() - timer, 2)}
    ctx.log.trace("finished transcription")

    result = {
        "text": vosk_result["text"],
        "parts": vosk_result["result"],
        "segments": segments,
        "meta": meta,
    }

    if recasepunc:
        ctx.log.trace("starting recasepunc")
        result = perform_recase(ctx.config, result)
        ctx.log.trace("finished recasepunc")

    return result

