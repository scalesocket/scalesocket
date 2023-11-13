# syntax=docker/dockerfile:1.2
FROM scalesocket/scalesocket:latest
RUN apk add --no-cache python3

WORKDIR /app

COPY index.html client.js bunny.png /var/www/public/
COPY server.py .

CMD scalesocket --addr 0.0.0.0:5000\
    --staticdir /var/www/public/\
    --json\
    --api\
    --metrics\
    /app/server.py
