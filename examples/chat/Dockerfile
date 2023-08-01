# syntax=docker/dockerfile:1.2
FROM scalesocket/scalesocket:latest
COPY index.html /var/www/public/index.html
CMD scalesocket --addr 0.0.0.0:5000\
    --staticdir /var/www/public/\
    --frame=json\
    --stats\
    cat
