# https://dev.to/sergeyzenchenko/actix-web-in-docker-how-to-build-small-and-secure-images-2mjd
FROM rust:1.81.0 AS build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/philologus-actix-web
COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian12

COPY --from=build /usr/local/cargo/bin/philologus-actix-web /usr/local/bin/philologus-actix-web

ENV PHILOLOGUS_DB_PATH=philolog_us.sqlite
ENV PHILOLOGUS_LOG_DB_PATH=philolog_us_log.sqlite

CMD ["philologus-actix-web"]
