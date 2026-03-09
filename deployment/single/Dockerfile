# =============================================================================
# Semaphore UI - Единый Docker контейнер (All-in-One)
# =============================================================================
# Этот Dockerfile создаёт самодостаточный образ с:
# - Rust backend (Axum)
# - Vue.js frontend (собранный)
# - Nginx (раздача статики + reverse proxy)
# - SQLite (встроенная БД, не требует отдельного контейнера)
#
# Использование:
#   docker build -f deployment/single/Dockerfile -t semaphore:latest .
#   docker run -d --name semaphore -p 80:80 -v semaphore_data:/app/data semaphore:latest
# =============================================================================

# -----------------------------------------------------------------------------
# Этап 1: Сборка backend (Rust)
# -----------------------------------------------------------------------------
FROM rust:1.88-slim AS backend-builder

WORKDIR /app

# Установка зависимостей для сборки (OpenSSL и др.)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Копирование Cargo файлов для кэширования зависимостей
COPY rust/Cargo.toml rust/Cargo.lock ./

# Создание пустого проекта для кэширования зависимостей
# Временный lib.rs для компиляции зависимостей
RUN mkdir -p src && echo "pub fn dummy() {}" > src/lib.rs && \
    cargo build --release && \
    rm -rf src target

# Копирование исходного кода Rust
COPY rust/src ./src

# Сборка проекта
RUN cargo build --release

# -----------------------------------------------------------------------------
# Этап 2: Сборка frontend (Vue.js)
# -----------------------------------------------------------------------------
FROM node:18-alpine AS frontend-builder

WORKDIR /web

# Копирование package файлов для кэширования зависимостей
COPY web/package.json web/package-lock.json ./

# Установка зависимостей
RUN npm ci

# Копирование исходного кода и сборка
COPY web/ ./
RUN npm run build

# Копирование собранного frontend в public для совместимости
RUN cp -r /web/dist /web/public-final

# -----------------------------------------------------------------------------
# Этап 3: Финальный образ
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim

# Метки образа
LABEL maintainer="Alexander Vashurin <alexandervashurin@yandex.ru>"
LABEL description="Semaphore UI - современный веб-интерфейс для управления DevOps инструментами"
LABEL version="0.1.0"
LABEL stack="Rust Axum + Vue.js Vuetify + Nginx + SQLite"

# Установка зависимостей
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    nginx \
    curl \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /var/www/html/* \
    && pip3 install bcrypt --break-system-packages

# Создание пользователя для запуска от непривилегированного аккаунта
RUN useradd -m -u 1000 semaphore

# Копирование бинарного файла из builder
COPY --from=backend-builder /app/target/release/semaphore /usr/local/bin/semaphore

# Копирование frontend из builder
COPY --from=frontend-builder /web/public-final /var/www/html

# Копирование конфигурации Nginx (полная конфигурация, не server block)
COPY deployment/single/nginx.conf /etc/nginx/nginx.conf

# Копирование скрипта инициализации
COPY deployment/single/init-admin.py /app/init-admin.py

# Создание директорий для данных
RUN mkdir -p /app/data /app/config /app/logs && \
    chown -R semaphore:semaphore /app /var/www/html /var/log/nginx /var/lib/nginx && \
    chmod -R 755 /app /var/www/html /var/log/nginx /var/lib/nginx

# Переключение на пользователя semaphore
USER semaphore

# Рабочая директория
WORKDIR /app

# Порты:
# 3000 - backend API
# 80 - frontend (Nginx)
EXPOSE 3000 80

# Переменные окружения
ENV SEMAPHORE_DB_URL="sqlite://data/semaphore.db"
ENV SEMAPHORE_WEB_PATH="/var/www/html"
ENV SEMAPHORE_HOST="0.0.0.0"
ENV SEMAPHORE_PORT="3000"
ENV RUST_LOG="info"

# Healthcheck
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Запуск: backend + инициализация + nginx
# Сначала запускаем сервер для создания БД, потом инициализируем пользователя
CMD ["sh", "-c", "echo 'Запуск Semaphore UI...' && echo 'Backend: http://0.0.0.0:3000' && echo 'Frontend: http://0.0.0.0:80' && (/usr/local/bin/semaphore server --host ${SEMAPHORE_HOST} --port ${SEMAPHORE_PORT} &) && sleep 3 && python3 /app/init-admin.py && nginx -g 'daemon off;'"]
