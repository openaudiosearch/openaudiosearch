# Note on running this with German models:
# Due to the way symbols are handled in Kaldi/PyKaldi, if you're using a non-English model
# (e.g., German), you have to set the right locales to obtain the correct output.
# An example for Debian-based distros is given below:
#
# apt-get install -y locales
# locale-gen de_DE.UTF-8
# export LC_ALL=de_DE.UTF-8
# export LANG=de_DE.UTF-8


from kaldi.base import set_verbose_level
from kaldi.segmentation import NnetSAD, SegmentationProcessor
from kaldi.asr import NnetLatticeFasterRecognizer
from kaldi.matrix import Matrix, SubMatrix
from kaldi.decoder import LatticeFasterDecoderOptions
from kaldi.fstext import SymbolTable, shortestpath, indices_to_symbols
from kaldi.fstext.utils import get_linear_symbol_sequence
from kaldi.nnet3 import NnetSimpleComputationOptions
from kaldi.feat.online import OnlineMatrixFeature
from kaldi.lat.align import (
    WordBoundaryInfo,
    WordBoundaryInfoNewOpts,
    word_align_lattice
)
from kaldi.lat.functions import compact_lattice_to_word_alignment
from kaldi.online2 import (
    OnlineIvectorExtractionInfo,
    OnlineIvectorExtractionConfig,
    OnlineIvectorFeature,
    OnlineIvectorExtractorAdaptationState,
    OnlineEndpointConfig
)
from kaldi.feat.mfcc import Mfcc, MfccOptions
from kaldi.util.options import ParseOptions
from kaldi.util.table import SequentialWaveReader
from kaldi.cudamatrix import cuda_available
if cuda_available():
    from kaldi.cudamatrix import CuDevice
    CuDevice.instantiate().select_gpu_id('yes')
    CuDevice.instantiate().allow_multithreading()

import wave
from typing import List, Tuple
import os

sad = None
asr = None

class Segmenter:
    def __init__(self, model_folder: os.PathLike, frame_subsampling_factor=3):
        """Segmenter or Speech activity detector class.

        Currently uses hard-coded values for the ASpIRE SAD model
        (http://kaldi-asr.org/models/m4). Will be made more configurable.

        Args:
            model_folder: Path to folder that contains Kaldi model files.
                NOTE: Currently uses approximately the format used by Vosk;
                however, one config file (ivector_extractor.conf) needs to be added.
                Vosk handles this by hard-coding the values in C++. This is not
                possible in pykaldi, so the configuration has to be read from a file.
            frame_subsampling_factor: Factor at which the rate of the output is lower
                than at the input of the neural network. Typically, this is 3 for
                so-called "chain" models (Kaldi's lattice-free maximum mutual information).
        """
        self.frame_shift = 0.01 * frame_subsampling_factor
        mfcc_opts = MfccOptions()
        mfcc_opts.use_energy = False   # use average of log energy, not energy.
        mfcc_opts.frame_opts.samp_freq = 8000
        mfcc_opts.mel_opts.num_bins = 40
        mfcc_opts.num_ceps = 40
        mfcc_opts.mel_opts.low_freq = 40
        mfcc_opts.mel_opts.high_freq = -200
        mfcc_opts.frame_opts.allow_downsample = True
        self.mfcc = Mfcc(mfcc_opts)

        # Construct SAD
        model = NnetSAD.read_model(os.path.join(model_folder, "segmentation/final.raw"))
        post = NnetSAD.read_average_posteriors(os.path.join(model_folder, "segmentation/post_output.vec"))
        transform = NnetSAD.make_sad_transform(
            post,
            sil_scale=0.1,
            sil_in_speech_weight=0.0,
            speech_in_sil_weight=0.0,
            garbage_in_speech_weight=0.0,
            garbage_in_sil_weight=0.0
        )
        # Construct SAD decoding graph
        graph = NnetSAD.make_sad_graph(
            min_silence_duration=0.03,
            min_speech_duration=0.3,
            max_speech_duration=10.0,
            frame_shift=self.frame_shift
        )
        decodable_opts = NnetSimpleComputationOptions()
        decodable_opts.extra_left_context = 79
        decodable_opts.extra_right_context = 21
        decodable_opts.extra_left_context_initial = 0
        decodable_opts.extra_right_context_final = 0
        decodable_opts.frame_subsampling_factor = frame_subsampling_factor
        decodable_opts.frames_per_chunk = 150
        decodable_opts.acoustic_scale = 0.3
        self.sad = NnetSAD(model, transform, graph, decodable_opts=decodable_opts)
        # Create segment post-processor
        self.seg = SegmentationProcessor(
            target_labels=[2],
            frame_shift=self.frame_shift,
            segment_padding=self.frame_shift * 6 # Heuristic; needs to be divisible by frame_shift
        )

    def process(self, audio_file_path: os.PathLike) -> List[Tuple[int, int, int]]:
        """
        Detect speech segments in an audio file

        Args:
            audio_file_path: Path to audio file.
        Returns:
            List of tuples of floats with segment (start, end) times in seconds.
        """
        with SequentialWaveReader(f"scp:echo utterance-id1 {audio_file_path}|") as reader:
            key, wav = next(iter(reader))
            # Get audio signal as vector
            signal = wav.data().row(0)
            # compute MFCC features
            feats = self.mfcc.compute_features(signal, wav.samp_freq, 1.0)
            # Run SAD model
            out = self.sad.segment(feats)
            # Get segments
            segments, _ = self.seg.process(out["alignment"])
            # Segments are given in terms of frame indices.
            # Here, we convert them to times in seconds.
            time_segments = []
            for start, end, _ in segments:
                time_segments.append(
                    (start*self.frame_shift, end*self.frame_shift)
                )
            return time_segments


class SpeechRecognizer:
    def __init__(self, model_folder):
        """
        Speech recognizer class using Kaldi Nnet3 and i-vectors.

        Args:
            model_folder: Path to folder that contains Kaldi model files.
                NOTE: Currently uses approximately the format used by Vosk;
                however, one config file (ivector_extractor.conf) needs to be added.
                Vosk handles this by hard-coding the values in C++. This is not
                possible in pykaldi, so the configuration has to be read from a file.
        """
        # Load MFCC feature extraction options
        mfcc_opts = MfccOptions()
        po = ParseOptions("")
        mfcc_opts.register(po)
        po.read_config_file(os.path.join(model_folder, "conf/mfcc.conf"))
        # Create MFCC feature extractor
        self.mfcc = Mfcc(mfcc_opts)

        # Load options for decoder
        decoder_opts = LatticeFasterDecoderOptions()
        decodable_opts = NnetSimpleComputationOptions()
        endpoint_opts = OnlineEndpointConfig()
        po = ParseOptions("")
        decoder_opts.register(po)
        decodable_opts.register(po)
        endpoint_opts.register(po)
        po.read_config_file(os.path.join(model_folder, "conf/model.conf"))

        # Create Speech Recognizer
        self.asr = NnetLatticeFasterRecognizer.from_files(
            os.path.join(model_folder, "am/final.mdl"),
            os.path.join(model_folder, "graph/HCLG.fst"),
            decoder_opts=decoder_opts,
            decodable_opts=decodable_opts
        )
        # Construct symbol table
        self.symbols = SymbolTable.read_text(os.path.join(model_folder, "graph/words.txt"))
        # Construct word boundary info (for word begin/end times)
        self.word_boundary_info = WordBoundaryInfo.from_file(
            WordBoundaryInfoNewOpts(),
            os.path.join(model_folder, "graph/phones/word_boundary.int")
        )

        # Construct Ivector Extractor config
        self.ivector_config = OnlineIvectorExtractionConfig()
        self.ivector_config.splice_config_rxfilename = os.path.join(model_folder, "ivector/splice.conf")
        self.ivector_config.cmvn_config_rxfilename = os.path.join(model_folder, "ivector/online_cmvn.conf")
        self.ivector_config.lda_mat_rxfilename = os.path.join(model_folder, "ivector/final.mat")
        self.ivector_config.global_cmvn_stats_rxfilename = os.path.join(model_folder, "ivector/global_cmvn.stats")
        self.ivector_config.diag_ubm_rxfilename = os.path.join(model_folder, "ivector/final.dubm")
        self.ivector_config.ivector_extractor_rxfilename = os.path.join(model_folder, "ivector/final.ie")

    def process_segments(self, audio_file_path: os.PathLike, segments: List[Tuple[float, float]]) -> List[str]:
        """
        Recognize speech in segments of an audio file.

        Args:
            audio_file_path: Path to audio file.
            segments: List of tuples of floats with segment (start, end) times in seconds.
        Returns:
            List of recognized words.
        """
        words = []
        word_ids = []
        begin_times = []
        end_times = []
        confidences = []
        with SequentialWaveReader(f"scp:echo utterance-id1 {audio_file_path}|") as reader:
            for key, wav in reader:
                # Get audio signal as vector
                signal = wav.data().row(0)

                # compute MFCC features
                feats = self.mfcc.compute_features(signal, wav.samp_freq, 1.0)

                # ivector extraction
                ivector_info = OnlineIvectorExtractionInfo.from_config(self.ivector_config)
                adaptation_state = OnlineIvectorExtractorAdaptationState.from_info(ivector_info)

                for seg_start_time, seg_end_time in segments:
                    seg_start_idx = int(seg_start_time/0.01)
                    seg_end_idx = min(int(seg_end_time/0.01), feats.num_rows)
                    seg_feats = SubMatrix(
                        feats,
                        row_start=seg_start_idx,
                        num_rows=seg_end_idx-seg_start_idx
                    )

                    matrix_feature = OnlineMatrixFeature(seg_feats)
                    ivector_feature = OnlineIvectorFeature(ivector_info, matrix_feature)
                    ivector_feature.set_adaptation_state(adaptation_state)
                    num_ivectors = int((seg_feats.num_rows + self.ivector_config.ivector_period - 1) / self.ivector_config.ivector_period)
                    ivectors = Matrix(num_ivectors, ivector_feature.dim())
                    # Get i-vectors
                    for i in range(num_ivectors):
                        t = int(i * self.ivector_config.ivector_period)
                        ivector_feature.get_frame(t, ivectors[i])

                    # Speech decoding
                    out = self.asr.decode((seg_feats, ivectors))
                    _, lattice = word_align_lattice(
                        out["best_path"],
                        self.asr.transition_model,
                        self.word_boundary_info,
                        max_states=0
                    )
                    seg_word_ids, begin_frames, duration_frames = compact_lattice_to_word_alignment(lattice)
                    likelihood = -(out["weight"].value1 + out["weight"].value2)
                    for word_id, begin, duration in zip(seg_word_ids, begin_frames, duration_frames):
                        word_ids.append(word_id)
                        begin_times.append(seg_start_time + begin * 0.03)
                        end_times.append(seg_start_time + (begin + duration) * 0.03)
                        confidences.append(likelihood)
                words = indices_to_symbols(self.symbols, word_ids)
        return words, begin_times, end_times, confidences

    def process(self, audio_file_path) -> List[str]:
        """
        Recognize speech in an entire audio file.

        Args:
            audio_file_path: Path to audio file.
        Returns:
            List of recognized words.
        """
        with SequentialWaveReader(f"scp:echo utterance-id1 {audio_file_path}|") as reader:
            for key, wav in reader:
                # Get audio signal as vector
                signal = wav.data().row(0)

                # compute MFCC features
                feats = self.mfcc.compute_features(signal, wav.samp_freq, 1.0)

                # ivector extraction
                ivector_info = OnlineIvectorExtractionInfo.from_config(self.ivector_config)
                adaptation_state = OnlineIvectorExtractorAdaptationState.from_info(ivector_info)
                matrix_feature = OnlineMatrixFeature(feats)
                ivector_feature = OnlineIvectorFeature(ivector_info, matrix_feature)
                ivector_feature.set_adaptation_state(adaptation_state)
                num_ivectors = int((feats.num_rows + self.ivector_config.ivector_period - 1) / self.ivector_config.ivector_period)
                ivectors = Matrix(num_ivectors, ivector_feature.dim())
                # Get i-vectors
                for i in range(num_ivectors):
                    t = int(i * self.ivector_config.ivector_period)
                    ivector_feature.get_frame(t, ivectors[i])

                # Speech decoding
                out = self.asr.decode((feats, ivectors))
                # Get most likely sequence of word IDs from lattice
                word_ids, _, _ = get_linear_symbol_sequence(shortestpath(out["lattice"]))
                # Convert word IDs to words
                words = indices_to_symbols(self.symbols, word_ids)
                return words


def transcribe_pykaldi(media_id, audio_file_path, model_folder):
    global sad
    if not sad:
        sad = Segmenter(model_folder)

    global asr
    if not asr:
        asr = SpeechRecognizer(model_folder)

    parts = []
    transcript = ""

    with wave.open(audio_file_path, "rb") as wf:
        # print(
        #     f'WAVE INFO chan {wf.getnchannels()} sampw {wf.getsampwidth()} comptype {wf.getcomptype()}')
        if wf.getnchannels() != 1 or wf.getsampwidth() != 2 or wf.getcomptype() != "NONE":
            raise ValueError('Audio file must be WAV format mono PCM.')

    segments = sad.process(audio_file_path)
    words, begin_times, end_times, confidences = asr.process_segments(audio_file_path, segments)
    transcript = " ".join(words)
    for word, begin_time, end_time, confidence in zip(words, begin_times, end_times, confidences):
        parts.append({
            "word": word,
            "start": begin_time,
            "end": end_time,
            "conf": confidence
        })

    return {'text': transcript, 'parts': parts }
