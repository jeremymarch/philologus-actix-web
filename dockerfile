# https://docs.docker.com/engine/install/ubuntu/
# https://docs.docker.com/engine/install/linux-postinstall/
# https://dev.to/sergeyzenchenko/actix-web-in-docker-how-to-build-small-and-secure-images-2mjd
# https://learn.arm.com/learning-paths/servers-and-cloud-computing/from-iot-to-the-cloud-part1/how-to-8/

FROM rust:1.81.0 AS build
# ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/philologus-actix-web
COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian12

COPY --from=build /usr/local/cargo/bin/philologus-actix-web /usr/local/bin/philologus-actix-web
COPY --from=build /usr/src/philologus-actix-web/db.sqlite /usr/local/bin/db.sqlite
COPY --from=build /usr/src/philologus-actix-web/static/ /usr/local/bin/static/
COPY --from=build /usr/src/philologus-actix-web/tantivy-data/ /usr/local/bin/tantivy-data/

ENV PHILOLOGUS_DB_PATH=sqlite:///usr/local/bin/db.sqlite?mode=ro
# ENV PHILOLOGUS_LOG_DB_PATH=sqlite://log.sqlite?mode=rwc
ENV TANTIVY_INDEX_PATH=/usr/local/bin/tantivy-data

# nb still need to call docker run with -p8088:8088
EXPOSE 8088

# set working directory for web server
# this must be set so web server can find static folder
# set here or set working directory with: docker run -w/usr/local/bin
WORKDIR /usr/local/bin

CMD ["philologus-actix-web"]


# docker build -t philologus-actix-web .

# to save hd space
# docker builder prune -a
# or
# docker system prune -a

# docker run [-w/usr/local/bin] -p8088:8088 -it philologus-actix-web
#   -w sets working directory, so actix-web can find static dir, but we can just use WORKDIR /usr/local/bin
#   -p8088:8088 opens port/maps to outside container


# https://stackoverflow.com/questions/78897082/difference-between-docker-buildx-build-and-docker-build-for-multi-arch-images
# docker buildx create --name multi-platform-builder --driver docker-container --use
# docker build --load --builder multi-platform-builder --platform=linux/amd64 -t philologus-actix-web .

# or multiple targets: docker build --load --builder multi-platform-builder --platform=linux/amd64,linux/arm64 -t philologus-actix-web .
