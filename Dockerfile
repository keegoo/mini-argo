# syntax=docker/dockerfile:1

# ---- build stage ----
FROM rust:1.97.1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src/ src/

RUN cargo build --release --locked

# ---- runtime stage ----
FROM debian:bookworm-slim

# ca-certificates: needed by rustls when verifying TLS certificates.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Run as a non-root user.
RUN useradd --system --uid 1000 --home /nonexistent --shell /usr/sbin/nologin mini-argocd

COPY --from=builder /app/target/release/mini-argocd-controller /usr/local/bin/mini-argocd-controller

USER mini-argocd
ENTRYPOINT ["/usr/local/bin/mini-argocd-controller"]
