FROM rustlang/rust:nightly-alpine3.12 as build

RUN apk add --no-cache musl-dev

COPY src /build/src
COPY Cargo.toml /build/
COPY config.yaml.example /build/config.yaml

WORKDIR /build
RUN cargo build --release

FROM alpine

RUN apk add ca-certificates

COPY --from=build /build/target/release/wednesday /opt/wednesday/
COPY --from=build /build/config.yaml /opt/wednesday/

WORKDIR /opt/wednesday
CMD ["/opt/wednesday/wednesday", "2>&1"]