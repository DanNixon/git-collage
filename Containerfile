FROM docker.io/library/debian:12-slim

RUN apt-get update \
      && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      libssl-dev \
      && \
    rm -rf /var/lib/apt/lists/*
