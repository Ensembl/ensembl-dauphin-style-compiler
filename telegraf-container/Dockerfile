# syntax=docker/dockerfile:1
FROM ubuntu:hirsute

RUN apt-get update && apt-get install -y bash telegraf

RUN mkdir -p /usr/local/etc
COPY etc/telegraf.conf /etc/telegraf

WORKDIR /
CMD /usr/bin/telegraf 2>&1 | logger -n ${LOG_HOST} -P ${LOG_PORT}
