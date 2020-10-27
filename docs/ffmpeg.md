# ffmpeg

some useful ffmpeg commands

Cut mp3 to first 30 seconds:
```bash
ffmpeg -t 30 -i inputfile.mp3 -acodec copy outputfile.mp3
```
