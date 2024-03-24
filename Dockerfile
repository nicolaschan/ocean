FROM rust:alpine

RUN apk add build-base gcc musl-dev cmake g++ pkgconf gdk-pixbuf-dev glib-dev pango-dev at-spi2-core-dev gtk+3.0-dev glib-static cairo-static gdk-pixbuf

COPY . .

RUN cargo install --path .

ENTRYPOINT ["ocean"]
