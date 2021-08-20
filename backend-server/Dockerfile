# syntax=docker/dockerfile:1
FROM ubuntu:groovy-20210723

RUN apt-get update
RUN apt-get install -y python3.8-venv pip libcurl4 libcurl4-openssl-dev memcached git

# need to run everything in venv as debian have disabled self-upgrade pip.
# we need to upgrade pip due to new git+https: support.

RUN pip install uvicorn fastapi loguru cbor2 toml pyaml numpy
# pybigwig needs numpy and curl libs installed before install is attempted
RUN pip install pybigwig pymemcache
RUN pip install --upgrade pip
RUN  pip install git+https://github.com/ens-ds23/ncd-python.git

WORKDIR /app
RUN rm -rf /app
COPY . /app
RUN python3 -m venv /app/venv

ENV DATA_FILES=/data
CMD . /app/venv/bin/activate && exec ./build.sh
