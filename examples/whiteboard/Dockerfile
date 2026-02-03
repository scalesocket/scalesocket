# syntax=docker/dockerfile:1.2
FROM node:25-alpine

COPY --from=scalesocket/scalesocket:latest /usr/bin /usr/bin

COPY index.html frontend.js images/ /var/www/public/
COPY server.ts /app/server.ts

WORKDIR /app
CMD scalesocket\
    --staticdir /var/www/public/\
    --json\
    node -- server.ts
