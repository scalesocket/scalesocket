# syntax=docker/dockerfile:1.2
FROM alpine:3.18
RUN apk add --no-cache tini curl

ARG VERSION 
RUN curl -SL "https://github.com/scalesocket/scalesocket/releases/download/${VERSION}/scalesocket_${VERSION}_x86_64-unknown-linux-musl.tar.gz" | tar -xzC /usr/bin

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["scalesocket", "--help"]
