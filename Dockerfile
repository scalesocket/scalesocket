# syntax=docker/dockerfile:1.2
ARG VERSION

FROM alpine:3.18 as base
ARG VERSION
RUN apk add --no-cache tini curl

FROM base AS build-arm64
ARG VERSION
RUN curl -SL "https://github.com/scalesocket/scalesocket/releases/download/${VERSION}/scalesocket_${VERSION}_aarch64-unknown-linux-musl.tar.gz" | tar -xzC /usr/bin

FROM base AS build-amd64
ARG VERSION
RUN curl -SL "https://github.com/scalesocket/scalesocket/releases/download/${VERSION}/scalesocket_${VERSION}_x86_64-unknown-linux-musl.tar.gz" | tar -xzC /usr/bin

FROM build-${TARGETARCH} AS build

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["scalesocket", "--help"]
