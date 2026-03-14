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

# «Пустая» сборка для прогрева кэша зависимостей
# Создаём заглушки для всех бинарей и бенчей указанных в Cargo.toml
RUN mkdir -p src benches \
    && echo "fn main() {}" > src/main.rs \
    && echo "" > src/lib.rs \
    && echo "fn main() {}" > benches/cache_bench.rs \
    && echo "fn main() {}" > benches/db_bench.rs \
    && cargo build --release \
    && rm -rf src benches

# ── Основная сборка ───────────────────────────────────────────────────────
FROM deps AS builder

COPY rust/ ./

# Инвалидируем кэш зависимостей если изменились исходники
RUN touch src/main.rs

# profile.release уже содержит: strip=true, lto=true, opt-level="z", panic=abort
RUN cargo build --release

# ── Финальный образ (~20 MB base + stripped binary) ───────────────────────
# gcr.io/distroless/cc-debian12:nonroot содержит glibc + libssl + ca-certs,
# работает с динамически слинкованными Rust бинарями без shell / apt.
# nonroot variant: UID=65532, GID=65532
FROM gcr.io/distroless/cc-debian12:nonroot

# Бинарь (уже stripped благодаря profile.release)
COPY --from=builder /app/target/release/semaphore /usr/local/bin/semaphore

# Vanilla JS фронтенд (если собран: npm run vanilla:build)
COPY --chown=65532:65532 web/public /app/web/public

WORKDIR /app

EXPOSE 3000

ENV SEMAPHORE_DB_URL="postgres://semaphore:semaphore_pass@db:5432/semaphore"
ENV SEMAPHORE_WEB_PATH=/app/web/public
ENV SEMAPHORE_ADMIN=admin
ENV SEMAPHORE_ADMIN_PASSWORD=demo123
ENV SEMAPHORE_ADMIN_NAME=Administrator
ENV SEMAPHORE_ADMIN_EMAIL=admin@semaphore.local
ENV SEMAPHORE_DEMO_MODE=true

CMD ["/usr/local/bin/semaphore", "server", "--host", "0.0.0.0", "--port", "3000"]
