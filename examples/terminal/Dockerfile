# syntax=docker/dockerfile:1.2
FROM scalesocket/scalesocket:latest
RUN apk add --no-cache\
    --repository http://dl-cdn.alpinelinux.org/alpine/edge/main\
    --repository http://dl-cdn.alpinelinux.org/alpine/edge/testing\
    expect vim

COPY index.html /var/www/public/index.html

CMD scalesocket\
    --binary\
    --staticdir /var/www/public/\
    unbuffer -- -p vim
