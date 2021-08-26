# syntax=docker/dockerfile:1
FROM nginx

RUN apt-get update && apt-get install -y nginx

RUN mkdir -p /usr/local/etc
COPY etc/nginx.conf.template /usr/local/etc

WORKDIR /

CMD usermod -u ${FILE_OWNER} nginx && groupmod -g${FILE_GROUP} nginx && /bin/bash -c 'envsubst "\$UPSTREAM \$LOG_HOST \$LOG_PORT \$NGINX_CACHE_SIZE" < /usr/local/etc/nginx.conf.template > /usr/local/etc/nginx.conf' && exec nginx -c /usr/local/etc/nginx.conf
