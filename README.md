# 🦀 Semaphore UI на Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-475%20passed-brightgreen.svg)]()
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Migration](https://img.shields.io/badge/migration-100%25-brightgreen.svg)]()
[![Frontend](https://img.shields.io/badge/frontend-Vue%202-brightgreen.svg)]()
[![Production Ready](https://img.shields.io/badge/status-production%20ready-brightgreen.svg)]()

**Полная миграция Semaphore UI на Rust** - высокопроизводительная, безопасная и надёжная система автоматизации для Ansible, Terraform, OpenTofu, Terragrunt, PowerShell и других DevOps-инструментов.

## 🎯 CRUD Демо

> **Попробуйте прямо сейчас!** Интерактивное демо с полным CRUD для всех сущностей.

```bash
# Быстрый старт
./demo-start.sh

# Откройте в браузере
http://localhost/demo-crud.html
```

**Учётные данные:**
- `admin` / `demo123` (администратор)
- `john.doe` / `demo123` (менеджер)
- `jane.smith` / `demo123` (менеджер)
- `devops` / `demo123` (исполнитель)

📖 **Подробная документация**: [CRUD_DEMO.md](CRUD_DEMO.md)

---

## 🚀 Быстрый Старт

### Требования

- Rust 1.75 или новее
- Cargo
- Docker (опционально, для PostgreSQL)

---

### Вариант 1: Docker (Frontend + PostgreSQL + демо-данные)

```bash
# 1. Запуск frontend и БД
./start.sh

# 2. Запуск backend (в отдельном терминале)
./start.sh --backend
```

**Доступ:** http://localhost

**Учётные данные:**
- `admin` / `demo123`
- `john.doe` / `demo123`
- `jane.smith` / `demo123`
- `devops` / `demo123`

---

### Вариант 2: SQLite (для тестирования)

```bash
# Сборка frontend
./web/build.sh

# Запуск сервера
cd rust
export SEMAPHORE_DB_PATH=/tmp/semaphore.db
cargo run -- server --host 0.0.0.0 --port 3000
```

**Доступ:** http://localhost:3000

**Создание администратора:**
```bash
cargo run -- user add --username admin --name "Admin" --email admin@localhost --password admin123 --admin
```

---

### Вариант 3: PostgreSQL (продакшен)

```bash
# 1. Запуск PostgreSQL
docker run -d --name semaphore-postgres \
  -e POSTGRES_USER=semaphore \
  -e POSTGRES_PASSWORD=semaphore_pass \
  -e POSTGRES_DB=semaphore \
  -p 5432:5432 \
  postgres:16-alpine

# 2. Сборка frontend
./web/build.sh

# 3. Запуск backend
cd rust
export SEMAPHORE_DB_URL="postgres://semaphore:semaphore_pass@localhost:5432/semaphore"
cargo run -- server --host 0.0.0.0 --port 3000
```

**Доступ:** http://localhost:3000

---

## 📚 Основные команды

```bash
# Запуск сервера
cargo run -- server --host 0.0.0.0 --port 3000

# Создание пользователя
cargo run -- user add --username <name> --email <email> --password <pwd> --admin

# Версия
cargo run -- version

# Тесты
cargo test
```

---

## 📖 Документация

| Документ | Описание |
|----------|----------|
| [CRUD_DEMO.md](CRUD_DEMO.md) | 🎯 CRUD Демо - полное руководство |
| [CONFIG.md](CONFIG.md) | Переменные окружения и конфигурация |
| [API.md](API.md) | REST API документация |
| [AUTH.md](AUTH.md) | Аутентификация и авторизация |
| [DOCKER_DEMO.md](DOCKER_DEMO.md) | Docker демонстрация |
| [scripts/README.md](scripts/README.md) | Скрипты запуска |

---

## 🛠 Технологический Стек

- **Backend:** Rust + Axum + SQLx
- **Frontend:** Vue 2
- **Базы данных:** SQLite, PostgreSQL, MySQL
- **Аутентификация:** JWT + bcrypt

---

## 📝 Лицензия

MIT © [Alexander Vashurin](https://github.com/alexandervashurin)

Оригинальный проект [Semaphore UI](https://github.com/semaphoreui/semaphore) на Go.
