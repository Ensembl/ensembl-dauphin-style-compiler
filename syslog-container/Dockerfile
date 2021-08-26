# syntax=docker/dockerfile:1
FROM ubuntu:groovy-20210723

RUN apt-get update && apt-get install -y syslog-ng

RUN mkdir -p /usr/local/etc
COPY etc/syslogd.conf /usr/local/etc

WORKDIR /

CMD touch /logs/syslog && chown $FILE_OWNER:$FILE_GROUP /logs/syslog && syslog-ng --no-caps -F -f /usr/local/etc/syslogd.conf
