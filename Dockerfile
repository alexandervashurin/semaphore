# Dockerfile для Semaphore UI (Rust backend)
# Использование: docker build -f Dockerfile -t semaphore-backend .

FROM rust:1.75-slim AS builder

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

# Финальный образ
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
ENV SEMAPHORE_DB_DIALECT=postgres
ENV SEMAPHORE_DB_HOST=db
ENV SEMAPHORE_DB_PORT=5432
ENV SEMAPHORE_DB_NAME=semaphore
ENV SEMAPHORE_DB_USER=semaphore
ENV SEMAPHORE_DB_PASS=semaphore123
ENV SEMAPHORE_WEB_PATH=/app/web/public

# Запуск приложения
CMD ["semaphore", "server", "--host", "0.0.0.0", "--port", "3000"]
