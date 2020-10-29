import logging

from app.core.job import Worker
from app.tasks.models import TranscribeOpts, TranscribeArgs
from app.worker import worker

if __name__ == '__main__':
  logging.basicConfig(level=logging.DEBUG)
  opts = TranscribeOpts(**{
    'engine': 'vosk'
  })

  url1 = 'https://audio1.thomann.de/wav_audiot/156266/2137_mp3-256.mp3?p=2x4p65p.mp3'
  url2 = 'https://arso.xyz'
  args1 = TranscribeArgs(**{
    'media_url': url1
  })
  args2 = TranscribeArgs(**{
    'media_url': url2
  })
  worker.enqueue_job('transcribe', args1, opts)
  worker.enqueue_job('transcribe', args2, opts)
  worker.run()