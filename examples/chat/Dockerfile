FROM scalesocket/scalesocket:latest
COPY index.html /var/www/public/index.html
CMD scalesocket\
    --json\
    --staticdir /var/www/public/\
    cat
