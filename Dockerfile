# syntax=docker/dockerfile:1
ARG VERSION

FROM alpine:3.23 AS download
ARG VERSION
RUN apk add --no-cache curl && \
    mkdir /arm64 /amd64 && \
    curl -SL "https://github.com/scalesocket/scalesocket/releases/download/${VERSION}/scalesocket_${VERSION}_aarch64-unknown-linux-musl.tar.gz" | tar -xzC /arm64 && \
    curl -SL "https://github.com/scalesocket/scalesocket/releases/download/${VERSION}/scalesocket_${VERSION}_x86_64-unknown-linux-musl.tar.gz" | tar -xzC /amd64

FROM alpine:3.23
ARG TARGETARCH
RUN apk add --no-cache tini
COPY --from=download /${TARGETARCH}/scalesocket /usr/bin/scalesocket
ENTRYPOINT ["/sbin/tini", "--"]
CMD ["scalesocket", "--help"]
