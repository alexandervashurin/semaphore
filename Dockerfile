# ============================================================================
# Dockerfile for Velum (Rust backend) - Debian musl cross-build + scratch runtime
# ============================================================================

FROM rust:1.90-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    musl-tools \
    pkg-config \
    libssl-dev \
    libssh2-1-dev \
    zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_STATIC=1
ENV OPENSSL_INCLUDE_DIR=/usr/include
ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
ENV LIBSSH2_SYS_USE_PKG_CONFIG=1
ENV CC_x86_64_unknown_linux_musl=musl-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc
ENV CFLAGS_x86_64_unknown_linux_musl="-I/usr/include/x86_64-linux-gnu"
ENV RUSTFLAGS="-C target-feature=+crt-static"

COPY rust/Cargo.toml rust/Cargo.lock ./rust/
COPY rust/build.rs ./rust/build.rs
COPY rust/benches ./rust/benches
COPY rust/proto ./rust/proto
COPY rust/src ./rust/src

WORKDIR /app/rust

RUN cargo build --locked --release --target x86_64-unknown-linux-musl --bin velum && \
    strip target/x86_64-unknown-linux-musl/release/velum

FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /app/rust/target/x86_64-unknown-linux-musl/release/velum /usr/local/bin/velum
COPY --chown=65532:65532 web/public /app/web/public

WORKDIR /app
USER 65532:65532

EXPOSE 3000

ENV SEMAPHORE_DB_DIALECT=postgres
ENV SEMAPHORE_WEB_PATH=/app/web/public
ENV SEMAPHORE_ADMIN=admin
ENV SEMAPHORE_ADMIN_PASSWORD=admin123
ENV SEMAPHORE_ADMIN_NAME=Administrator
ENV SEMAPHORE_ADMIN_EMAIL=admin@velum.local

HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=5 \
    CMD ["/usr/local/bin/velum", "healthcheck", "--url", "http://127.0.0.1:3000/healthz"]

CMD ["/usr/local/bin/velum", "server", "--host", "0.0.0.0", "--port", "3000"]
