# Installation

Installation Steps:
* miniconda
* Additional packages
* Conda environment
* Download models


### miniconda
Download miniconda installer (Python 3.8) from [here](https://docs.conda.io/en/latest/miniconda.html). Run the installer script `bash Miniconda3-latest-Linux-*.sh` in Terminal and follow the prompts. After installation is completed, re-open Terminal or `source ~/.bashrc`.

### Additional packages
Install additional packages on your machine:  
`sudo apt-get install portaudio19-dev`

### Conda environment
Creat a new conda environment, using the yaml file provided in `oas_core` directory.  
`conda env create -f environment.yml`

Activate environment: `conda activate oas`

### Download models
Run automatic download script  in `oas_core/`directory: ` python task-run.py download_models `.

Or, manually download and extract the following models repositories in `/tmp/oas/models`:

* [VOSK Standard DE](https://alphacephei.com/vosk/models/vosk-model-de-0.6.zip)
* [VOSK Speaker Identification](https://alphacephei.com/vosk/models/vosk-model-spk-0.4.zip)


# Update your installation

After pulling changes from others, you want to update your python environment.
Run this command to update the active conda environment with changes:
```bash
conda env update -f environment.yml
```