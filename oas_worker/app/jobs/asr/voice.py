
import time
import torch
torch.device("cpu")
# torch.set_num_threads(1)
# torch.multiprocessing.set_start_method('spawn')
# torch.device("cpu")


SAMPLE_RATE = 16000
USE_ONNX = False
# change this to True if you want to test onnx model
#  if USE_ONNX:
#      !pip install -q onnxruntime
  

vad_model = None
def get_model():
    global vad_model
    if not vad_model:
        model, utils = torch.hub.load(repo_or_dir='snakers4/silero-vad',
                                    model='silero_vad',
                                    #  force_reload=True,
                                    onnx=USE_ONNX)
        model.share_memory()
        vad_model = (model, utils)
    return vad_model

def get_voice_segments(wav, sample_rate=SAMPLE_RATE, optimize=True):
    (model, utils) = get_model()
    (
        get_speech_timestamps,
        save_audio,
        read_audio,
        VADIterator,
        collect_chunks
    ) = utils

    wav = read_audio(wav, sampling_rate=sample_rate)
    # get speech timestamps from full audio file
    speech_timestamps = get_speech_timestamps(wav, model, sampling_rate=sample_rate)
    if not optimize:
        return speech_timestamps
    else:
        return optimize_voice_segments(speech_timestamps, sample_rate=sample_rate)


def optimize_voice_segments (segments, sample_rate=SAMPLE_RATE, min_pause=2):
    result = []
    last = None
    for segment in segments:
        if not last:
            last = segment
            continue

        if last and segment["start"] < (last["end"] + min_pause * sample_rate):
            last["end"] = segment["end"]
        else:
            start = last["start"] / sample_rate
            end = last["end"] / sample_rate
            result.append({ "start": start, "end": end, "type": "voice" })
            last = None

    if last:
        start = last["start"] / sample_rate
        end = last["end"] / sample_rate
        result.append({ "start": start, "end": end, "type": "voice" })

    return result

def format_frames(frames, sample_rate):
    secs = frames / sample_rate
    fmt = time.strftime("%H:%M:%S",time.gmtime(secs))
    return fmt
