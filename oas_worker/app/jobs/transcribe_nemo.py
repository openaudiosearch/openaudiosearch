import math
import os
import torch
import nemo.collections.asr as nemo_asr
from nemo.collections.asr.parts.utils.streaming_utils import FrameBatchASR
from nemo.utils import logging
logging.setLevel(logging.ERROR)

asr_model = None
vad_model = None

def transcribe_nemo(media_id, audio_file_path, model_folder):
    global asr_model
    if not asr_model:
        asr_model = nemo_asr.models.EncDecCTCModelBPE.from_pretrained(
            model_name='stt_de_citrinet_1024'
        )
    total_buffer_in_secs = 4.0
    batch_size = 32
    chunk_len = 1.6
    model_stride = 8

    torch.set_grad_enabled(False)
    asr_model = nemo_asr.models.EncDecCTCModelBPE.restore_from(
        os.path.join(model_folder, 'stt_de_citrinet_1024.nemo')
    )

    asr_model.eval()
    asr_model = asr_model.to(asr_model.device)

    feature_stride = asr_model._cfg.preprocessor['window_stride']
    model_stride_in_secs = feature_stride * model_stride

    tokens_per_chunk = math.ceil(chunk_len / model_stride_in_secs)
    delay = math.ceil((chunk_len + (total_buffer_in_secs - chunk_len) / 2) / model_stride_in_secs)

    frame_asr = FrameBatchASR(
        asr_model=asr_model,
        frame_len=chunk_len,
        total_buffer=total_buffer_in_secs,
        batch_size=batch_size,
    )

    frame_asr.read_audio_file(audio_file_path, delay, model_stride_in_secs)
    transcript = frame_asr.transcribe(tokens_per_chunk, delay)
    return {'text': transcript, 'parts': transcript.split() }
