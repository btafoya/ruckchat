# syntax=docker/dockerfile:1

# Multi-stage Dockerfile for ruckchat-server.
#
# Build requirements:
#   - Web UI assets must be pre-built in web/dist/:
#       cd web && pnpm install && pnpm build
#   - SQLx offline query data must be present in .sqlx/ (run `cargo sqlx prepare`).
#
# Example build:
#   docker build -t ruckchat-server .

FROM rust:1.94-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/ruckchat

# Copy the workspace manifest and all crate manifests first to leverage Docker layer
# caching for dependency downloads.
COPY Cargo.toml .
COPY crates crates
COPY server server
COPY migrations migrations
COPY .sqlx .sqlx
COPY desktop/src-tauri desktop/src-tauri
COPY web web

ENV SQLX_OFFLINE=true

RUN cargo build -p ruckchat-server --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        libpq5 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r ruckchat && useradd -r -g ruckchat ruckchat

WORKDIR /app

COPY --from=builder /usr/src/ruckchat/target/release/ruckchat-server /usr/local/bin/ruckchat-server

# Default data directories. Operators should mount volumes here for persistence.
RUN mkdir -p /var/lib/ruckchat/plugins /var/lib/ruckchat/files /etc/ruckchat \
    && chown -R ruckchat:ruckchat /var/lib/ruckchat /etc/ruckchat /app

USER ruckchat

EXPOSE 3000

# Probe the port declared in base_url so the healthcheck stays valid when
# operators change the listening port in ruckchat.yaml. Defaults to 3000
# if no explicit port is found.
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD PORT=$(awk -F':' '/^base_url:/ {gsub(/[^0-9]/,"",$NF); print $NF}' /etc/ruckchat/ruckchat.yaml | head -n1); curl -fsS "http://localhost:${PORT:-3000}/" >/dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/ruckchat-server"]
CMD ["--config", "/etc/ruckchat/ruckchat.yaml"]
