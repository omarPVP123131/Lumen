# Dockerfile para LÚMEN — Multi-stage build
FROM rust:1.84-slim-bookworm AS builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y --no-install-recommends \
    libclang-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release --bin lumen

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/lumen /usr/local/bin/lumen
COPY stdlib/ /usr/local/share/lumen/stdlib/

RUN useradd -m -s /bin/bash lumen
USER lumen
WORKDIR /home/lumen

ENTRYPOINT ["lumen"]
CMD ["--help"]
