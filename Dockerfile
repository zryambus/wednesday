FROM alpine:latest

RUN apk --no-cache add ca-certificates && update-ca-certificates

COPY target/x86_64-unknown-linux-musl/release/wednesday /opt/wednesday/
COPY config.yaml.example /opt/wednesday/config.yaml

WORKDIR /opt/wednesday
CMD ["/opt/wednesday/wednesday", "2>&1"]
# ENTRYPOINT ["/opt/wednesday/wednesday", "2>&1"]
