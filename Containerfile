# Build
FROM docker.io/library/rust:latest as builder

WORKDIR /app
COPY . .

RUN cargo build --release

# Runtime
FROM docker.io/library/debian:13-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
    && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/git-collage /app/git-collage

WORKDIR /data
VOLUME /data

ENTRYPOINT ["/app/git-collage"]
