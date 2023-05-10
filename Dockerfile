# syntax=docker/dockerfile:1

FROM --platform=$BUILDPLATFORM rust:latest AS buildbase
WORKDIR /mysql-binlog-kafka
RUN <<EOT bash
    set -ex
    apt-get update
    apt-get install -y \
        git \
        clang
    rustup target add wasm32-wasi
EOT

FROM buildbase AS build
WORKDIR /mysql-binlog-kafka/mysql-binlog-kafka
COPY mysql-binlog-kafka/Cargo.toml .
COPY mysql-binlog-kafka/src ./src
WORKDIR /mysql-binlog-kafka
COPY mysql_cdc ./mysql_cdc
WORKDIR /mysql-binlog-kafka/mysql-binlog-kafka
# Build the Wasm binary
RUN cargo build --target wasm32-wasi --release

FROM scratch
ENTRYPOINT [ "mysql-binlog-kafka.wasm" ]
COPY --link --from=build /mysql-binlog-kafka/mysql-binlog-kafka/target/wasm32-wasi/release/mysql-binlog-kafka.wasm /mysql-binlog-kafka.wasm
