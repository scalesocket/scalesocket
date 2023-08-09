# syntax=docker/dockerfile:1.2
FROM scalesocket/scalesocket:latest
RUN apk add --no-cache\
    --repository http://dl-cdn.alpinelinux.org/alpine/edge/main\
    --repository http://dl-cdn.alpinelinux.org/alpine/edge/testing\
    expect nsnake

RUN adduser -D demo
USER demo

COPY index.html /var/www/public/index.html

CMD scalesocket --addr 0.0.0.0:5000\
    --binary\
    --staticdir /var/www/public/\
    --api\
    --metrics\
    # launch using unbuffer to expose a pty
    ash -- -c 'TERM=xterm-color unbuffer -p nsnake'
