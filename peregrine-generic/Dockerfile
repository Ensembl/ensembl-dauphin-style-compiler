# syntax=docker/dockerfile:1
FROM python:3.8

LABEL maintainer="ensembl-webteam@ebi.ac.uk"

ENV PYTHONUNBUFFERED TRUE

RUN apt-get update
RUN apt-get install -y curl

RUN rm -rf /usr/src/app
RUN mkdir /usr/src/app
COPY . /usr/src/app

WORKDIR /usr/src/app

CMD python3 server.py

