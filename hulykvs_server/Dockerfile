FROM rust:1.86 AS builder
WORKDIR /tmp/build
COPY . .
RUN cargo build --release

FROM debian:12-slim
LABEL org.opencontainers.image.source="https://github.com/aav/hulykvs"
COPY --from=builder /tmp/build/target/release/hulykvs /usr/local/bin/hulykvs
RUN apt-get update && apt-get install -y ca-certificates
ENTRYPOINT ["/usr/local/bin/hulykvs"]
