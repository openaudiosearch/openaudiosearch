from app.tasks.tasks import transcribe

args = {
    'media_url': 'foo',
    'media_id': 'bar'
}

opts = {}

transcribe(args, opts)
