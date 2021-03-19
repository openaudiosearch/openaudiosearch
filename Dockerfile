FROM continuumio/miniconda3
RUN apt-get update && apt-get install -q -y portaudio19-dev gcc
WORKDIR /code
COPY oas_core/ /code/
RUN conda env create -f environment.yml

# Make RUN commands use the new environment:
#SHELL ["conda", "run", "-n", "oas", "/bin/bash", "-c"]
#RUN python task-run.py download_models
#
#ENTRYPOINT ["conda", "run", "-n", "oas", "python", "server.py"]
