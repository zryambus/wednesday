FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates

COPY target/x86_64-unknown-linux-gnu/release/wednesday /opt/wednesday/
COPY config.yaml.example /opt/wednesday/config.yaml

WORKDIR /opt/wednesday
CMD ["/opt/wednesday/wednesday", "2>&1"]