# ============================================================================
# Dockerfile for Velum — optimized for <50MB (~23MB)
# Multi-stage: builder (glibc) → scratch runtime with minimal libs
# ============================================================================

# ---- Stage 1: Builder ----
FROM debian:bookworm-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    gcc \
    pkg-config \
    libssl-dev \
    libssh2-1-dev \
    zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY rust/Cargo.toml rust/Cargo.lock ./rust/
COPY rust/build.rs ./rust/build.rs
COPY rust/benches ./rust/benches
COPY rust/proto ./rust/proto
COPY rust/src ./rust/src
WORKDIR /app/rust

RUN cargo build --locked --release --bin velum && \
    strip target/release/velum

# ---- Stage 2: Extract libraries from builder ----
FROM debian:bookworm-slim AS libs

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libssh2-1 \
    zlib1g \
    && rm -rf /var/lib/apt/lists/*

# Copy all needed .so files to a single directory
RUN mkdir -p /out/lib && \
    cp /lib/x86_64-linux-gnu/libz.so.1 /out/lib/ && \
    cp /lib/x86_64-linux-gnu/libssl.so.3 /out/lib/ && \
    cp /lib/x86_64-linux-gnu/libcrypto.so.3 /out/lib/ && \
    cp /lib/x86_64-linux-gnu/libgcc-s.so.1 /out/lib/ && \
    cp /lib/x86_64-linux-gnu/libm.so.6 /out/lib/ && \
    cp /lib/x86_64-linux-gnu/libc.so.6 /out/lib/ && \
    cp /lib/x86_64-linux-gnu/libzstd.so.1 /out/lib/ && \
    cp /lib64/ld-linux-x86-64.so.2 /out/lib/ && \
    cp /lib/x86_64-linux-gnu/libdl.so.2 /out/lib/ 2>/dev/null || true && \
    cp /lib/x86_64-linux-gnu/libpthread.so.0 /out/lib/ 2>/dev/null || true && \
    cp /lib/x86_64-linux-gnu/librt.so.1 /out/lib/ 2>/dev/null || true

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /out/etc/ssl/certs/

# ---- Stage 3: Scratch runtime ----
FROM scratch

COPY --from=libs /out/lib /lib
COPY --from=libs /out/etc /etc
COPY --from=builder /app/rust/target/release/velum /app/velum
COPY --chown=0:0 web/public /app/web/public

EXPOSE 3000

ENV LD_LIBRARY_PATH=/lib
ENV SEMAPHORE_DB_DIALECT=postgres
ENV SEMAPHORE_WEB_PATH=/app/web/public
ENV SEMAPHORE_ADMIN=admin
ENV SEMAPHORE_ADMIN_PASSWORD=admin123
ENV SEMAPHORE_ADMIN_NAME=Administrator
ENV SEMAPHORE_ADMIN_EMAIL=admin@velum.local

ENTRYPOINT ["/app/velum"]
CMD ["server", "--host", "0.0.0.0", "--port", "3000"]
