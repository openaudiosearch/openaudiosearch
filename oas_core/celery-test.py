from app.tasks.tasks import download
from app.tasks.models import *
download.delay("https://rdl.de/sites/default/files/audio/2021/04/20210429-fokussdwest2-w23843.mp3?dl=1")
