# syntax=docker/dockerfile:1
FROM python:3.10-bookworm

LABEL maintainer="ensembl-webteam@ebi.ac.uk"

ENV PYTHONUNBUFFERED=true \
    PYTHONDONTWRITEBYTECODE=true \
    PIP_NO_CACHE_DIR=true

RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        build-essential \
        git \
        libcurl4 \
        libcurl4-openssl-dev \
        memcached \
        python3-distutils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY requirements.txt requirements2.txt ./

RUN python -m pip install --upgrade pip setuptools wheel \
    && python -m pip install -r requirements.txt \
    && python -m pip install -r requirements2.txt

COPY . .

WORKDIR /usr/src/app/app

EXPOSE 3333

CMD ["sh", "-c", "exec uvicorn main:app --host 0.0.0.0 --port 3333 --workers \"${THREADS:?THREADS must be set}\""]
