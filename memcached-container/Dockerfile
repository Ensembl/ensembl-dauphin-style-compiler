# syntax=docker/dockerfile:1
FROM ubuntu:jammy

RUN apt-get update && apt-get install -y memcached
RUN useradd memcached -s /bin/bash -m

WORKDIR /

CMD memcached -u memcached -m ${MEMCACHED_MB:-4096} -p 11211 -l 0.0.0.0
