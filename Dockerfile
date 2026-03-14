# ============================================================================
# Dockerfile для Semaphore UI (Rust backend)
# ============================================================================
# Использование:
#   docker build -f Dockerfile -t semaphore-backend .
#   docker run -p 3000:3000 semaphore-backend
#
# Демо-режим с тестовыми данными:
#   docker-compose -f docker-compose.postgres.yml up -d
# ============================================================================

FROM rust:1.80-slim AS builder

# Установка зависимостей для сборки
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Установка рабочей директории
WORKDIR /app

# Копирование Cargo файлов для кэширования зависимостей
COPY rust/Cargo.toml rust/Cargo.lock ./

# Создание пустого проекта для кэширования зависимостей
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target

# Копирование исходного кода
COPY rust/ ./

# Сборка проекта
RUN cargo build --release

# ============================================================================
# Финальный образ
# ============================================================================
FROM debian:bookworm-slim

# Установка зависимостей для запуска
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Создание пользователя для запуска от непривилегированного аккаунта
RUN useradd -m -u 1000 semaphore

# Копирование бинарного файла из builder
COPY --from=builder /app/target/release/semaphore /usr/local/bin/

# Копирование frontend (если собран)
COPY --chown=semaphore:semaphore web/public /app/web/public

# Рабочая директория
WORKDIR /app

# Переключение на пользователя semaphore
USER semaphore

# Порт приложения
EXPOSE 3000

# Переменные окружения по умолчанию
ENV SEMAPHORE_DB_URL="postgres://semaphore:semaphore_pass@db:5432/semaphore"
ENV SEMAPHORE_WEB_PATH=/app/web/public
ENV SEMAPHORE_ADMIN=admin
ENV SEMAPHORE_ADMIN_PASSWORD=demo123
ENV SEMAPHORE_ADMIN_NAME=Administrator
ENV SEMAPHORE_ADMIN_EMAIL=admin@semaphore.local
ENV SEMAPHORE_DEMO_MODE=true

# Запуск приложения
CMD ["semaphore", "server", "--host", "0.0.0.0", "--port", "3000"]
