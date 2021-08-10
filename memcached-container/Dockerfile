# syntax=docker/dockerfile:1
FROM ubuntu:groovy-20210723

RUN apt-get update && apt-get install -y memcached
RUN useradd memcached -s /bin/bash -m

WORKDIR /

CMD ["memcached","-u","memcached","-m","512","-p","11211","-l","0.0.0.0"]
