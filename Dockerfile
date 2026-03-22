# ============================================================================
# Dockerfile для Semaphore UI (Rust backend) — multi-stage, цель < 50 MB
# ============================================================================
# Использование:
#   docker build -f Dockerfile -t semaphore-backend .
#   docker run -p 3000:3000 semaphore-backend
# ============================================================================

# ── Зависимости (кэшируются отдельно от исходников) ──────────────────────
FROM rust:slim AS deps

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем только манифесты, чтобы слой зависимостей кэшировался
COPY rust/Cargo.toml rust/Cargo.lock ./

# ── Основная сборка ───────────────────────────────────────────────────────
FROM deps AS builder

COPY rust/ ./

# profile.release уже содержит: strip=true, lto=true, opt-level="z", panic=abort
RUN cargo build --release && mkdir -p /app/data

# ── Финальный образ (~20 MB base + stripped binary) ───────────────────────
# gcr.io/distroless/cc-debian12:nonroot содержит glibc + libssl + ca-certs,
# работает с динамически слинкованными Rust бинарями без shell / apt.
# nonroot variant: UID=65532, GID=65532
FROM gcr.io/distroless/cc-debian12:nonroot

# Бинарь (уже stripped благодаря profile.release)
COPY --from=builder /app/target/release/velum /usr/local/bin/velum

# Vanilla JS фронтенд
COPY --chown=65532:65532 web/public /app/web/public

# Директория для SQLite БД (создаётся через пустую папку из builder)
COPY --from=builder --chown=65532:65532 /app/data /app/data

WORKDIR /app

EXPOSE 3000

# SQLite по умолчанию — не нужна отдельная БД для запуска
ENV SEMAPHORE_DB_DIALECT=sqlite
ENV SEMAPHORE_DB_PATH=/app/data/semaphore.db
ENV SEMAPHORE_WEB_PATH=/app/web/public
# Демо-учётные данные — сид при первом запуске
ENV SEMAPHORE_ADMIN=admin
ENV SEMAPHORE_ADMIN_PASSWORD=admin123
ENV SEMAPHORE_ADMIN_NAME=Administrator
ENV SEMAPHORE_ADMIN_EMAIL=admin@semaphore.local

CMD ["/usr/local/bin/velum", "server", "--host", "0.0.0.0", "--port", "3000"]
