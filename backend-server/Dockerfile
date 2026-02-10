# syntax=docker/dockerfile:1
FROM python:3.10-bullseye AS build

LABEL maintainer="ensembl-webteam@ebi.ac.uk"

ENV PYTHONUNBUFFERED TRUE

RUN apt-get update
RUN apt-get install -y \
	python3-distutils \
	build-essential \
	libcurl4 \
	libcurl4-openssl-dev \
	memcached \
	git

RUN rm -rf /usr/src/app
RUN mkdir /usr/src/app
COPY requirements.txt /usr/src/app/
COPY requirements2.txt /usr/src/app/

RUN pip3 install --no-cache-dir --upgrade pip setuptools wheel
RUN pip3 install --no-cache-dir -r /usr/src/app/requirements.txt
RUN pip3 install --no-cache-dir -r /usr/src/app/requirements2.txt

FROM build

COPY . /usr/src/app

WORKDIR /usr/src/app/app

EXPOSE 3333

CMD uvicorn main:app --host 0.0.0.0 --port 3333 --workers $THREADS
