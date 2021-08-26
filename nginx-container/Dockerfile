# syntax=docker/dockerfile:1
FROM nginx

RUN apt-get update && apt-get install -y nginx python3

RUN mkdir -p /usr/local/etc
COPY etc/nginx.conf.template /usr/local/etc
COPY bin/nginx-wrapper.py /usr/local/bin

WORKDIR /

CMD usermod -u ${FILE_OWNER} nginx && groupmod -g${FILE_GROUP} nginx && \
    /bin/bash -c 'envsubst "\$UPSTREAM \$LOG_HOST \$LOG_PORT \$NGINX_CACHE_SIZE" < /usr/local/etc/nginx.conf.template > /usr/local/etc/nginx.conf' && \
    exec /usr/local/bin/nginx-wrapper.py -s "$LOG_HOST:$LOG_PORT" -r /cache/restart.now -p /tmp/nginx.pid -c /cache nginx -c /usr/local/etc/nginx.conf
