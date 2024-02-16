# syntax=docker/dockerfile:1.2
FROM node:20-alpine
RUN apk --no-cache add curl
RUN mkdir -p /app/node_modules && chown -R node:node /app

ENV SCALESOCKET_VERSION=v0.2.2
RUN curl -SL "https://github.com/scalesocket/scalesocket/releases/download/${SCALESOCKET_VERSION}/scalesocket_${SCALESOCKET_VERSION}_x86_64-unknown-linux-musl.tar.gz" | tar -xzC /usr/local/bin

WORKDIR /app
USER node

COPY --chown=node:node package*.json ./
RUN npm install

COPY index.html utils.js /var/www/public/

CMD scalesocket --addr 0.0.0.0:8000 --staticdir /var/www/public/ --json\
    npm -- run start