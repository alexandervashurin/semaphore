# Настройка окружения

> Руководство по настройке локального окружения для разработки Velum
>
> 📖 См. также: [Структура кода](./code-structure.md), [Руководство по тестированию](./testing.md), [Участие в проекте](../contributing/contributing.md)

---

## Требования

- **Rust** 1.80+ (установить через `rustup`)
- **PostgreSQL** 15+
- **pkg-config**, **libssl-dev**, **libsqlite3-dev**
- **Node.js** (для фронтенда, опционально)

## Быстрый старт разработки

```bash
# Клонирование
git clone https://github.com/alexandervashurin/semaphore.git
cd semaphore/rust

# Установка зависимостей
cargo build

# Запуск сервера
VELUM_DB_DIALECT=postgres \
VELUM_DB_URL="postgres://user:pass@localhost:5432/velum" \
VELUM_JWT_SECRET="dev-secret-key-32-bytes-long!!" \
VELUM_WEB_PATH="../web/public" \
cargo run -- server --host 0.0.0.0 --port 3000
```

## Режим разработки с горячей перезагрузкой

```bash
cargo install cargo-watch
cargo watch -x 'run -- server --host 0.0.0.0 --port 3000'
```

Или используйте Docker:

```bash
docker compose -f docker-compose.dev.yml up -d
```

См. [Docker](../deployment/docker-deployment.md)

---

## Следующие шаги

- [Структура кода](./code-structure.md) — организация модулей Rust
- [Руководство по тестированию](./testing.md) — написание и запуск тестов
- [Участие в проекте](../contributing/contributing.md) — процесс Pull Request
