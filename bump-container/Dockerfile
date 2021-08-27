# syntax=docker/dockerfile:1
FROM python:3.8

LABEL maintainer="ensembl-webteam@ebi.ac.uk"

ENV PYTHONUNBUFFERED TRUE

RUN apt-get update

COPY bin/bump-detector.py /usr/local/bin
COPY requirements.txt /usr/local/etc

RUN pip install --no-cache-dir -r /usr/local/etc/requirements.txt


WORKDIR /

CMD exec /usr/local/bin/bump-detector.py -s "$LOG_HOST:$LOG_PORT" -r /cache/restart.now -m $MEMCACHED -k bump $BUMP_URL
