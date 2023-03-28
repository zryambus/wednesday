FROM rust:slim-bullseye as build

RUN apt-get update && apt-get install -y build-essential

COPY src /build/src
COPY Cargo.toml /build/
COPY config.yaml.example /build/config.yaml

WORKDIR /build
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates

COPY --from=build /build/target/release/wednesday /opt/wednesday/
COPY --from=build /build/config.yaml /opt/wednesday/

WORKDIR /opt/wednesday
CMD ["/opt/wednesday/wednesday", "2>&1"]