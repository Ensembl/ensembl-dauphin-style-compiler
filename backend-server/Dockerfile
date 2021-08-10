# syntax=docker/dockerfile:1
FROM ubuntu:groovy-20210723

RUN apt-get update && apt-get install -y pip libcurl4 libcurl4-openssl-dev memcached

RUN pip install uvicorn fastapi loguru cbor2 toml pyaml numpy
# pybigwig needs numpy and curl libs installed before install is attempted
RUN pip install pybigwig pymemcache

WORKDIR /app
COPY . /app

ENV DATA_FILES=/data
CMD ["./build.sh"]
