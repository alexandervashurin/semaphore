# 🦀 Velum — Rust Edition

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.80+-blue.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-667%20passed-brightgreen.svg)]()
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Migration](https://img.shields.io/badge/migration-100%25-brightgreen.svg)]()
[![Frontend](https://img.shields.io/badge/frontend-Vanilla%20JS-brightgreen.svg)]()
[![Production Ready](https://img.shields.io/badge/status-production%20ready-brightgreen.svg)]()

**Полная миграция [Velum](https://github.com/velum/velum) с Go на Rust** — высокопроизводительная, безопасная и надёжная система автоматизации для Ansible, Terraform, OpenTofu, Terragrunt, PowerShell и других DevOps-инструментов.

---

## 🎯 Быстрый старт (Docker Demo)

> **Запустить прямо сейчас!** SQLite + автосид admin/admin123, порт 8088.

```bash
docker compose -f docker-compose.demo.yml up --build -d

# Откройте в браузере
http://localhost:8088
```

**Учётные данные:**
- `admin` / `admin123`

---

## 🚀 Запуск для разработки (SQLite, без Docker)

```bash
cd rust

# С SQLite
export SEMAPHORE_DB_DIALECT=sqlite
export SEMAPHORE_DB_PATH=/tmp/semaphore.db
cargo run -- server --host 0.0.0.0 --port 3000
```

**Создание администратора:**
```bash
cd rust
cargo run -- user add \
  --username admin \
  --name "Administrator" \
  --email admin@localhost \
  --password admin123 \
  --admin
```

**Доступ:** http://localhost:3000

---

## 🐘 Запуск с PostgreSQL

```bash
# Полный стек через Docker (PostgreSQL + backend)
docker compose up -d

# Или вручную
export SEMAPHORE_DB_DIALECT=postgres
export SEMAPHORE_DB_URL=postgres://semaphore:semaphore123@localhost:5432/semaphore
cd rust && cargo run -- server --host 0.0.0.0 --port 3000
```

---

## 📚 Основные команды разработки

```bash
# Проверка компиляции
cd rust && cargo check

# Линтер (0 warnings)
cd rust && cargo clippy -- -D warnings

# Тесты (667 тестов)
cd rust && cargo test

# Запуск сервера (SQLite)
cd rust && SEMAPHORE_DB_PATH=/tmp/semaphore.db cargo run -- server

# Создание пользователя
cargo run -- user add --username <name> --email <email> --password <pwd> --admin

# Версия
cargo run -- version
```

---

## 🔐 Шифрование ключей доступа (опционально)

```bash
export SEMAPHORE_ACCESS_KEY_ENCRYPTION="your-secret-passphrase"
```

Если переменная задана — все SSH/API ключи в БД шифруются AES-256-GCM. Без неё — хранятся в plaintext (как в оригинальном Go Velum).

---

## 📖 Документация

| Документ | Описание |
|----------|----------|
| [MASTER_PLAN.md](MASTER_PLAN.md) | 📋 Живой план миграции и статус задач |
| [CONFIG.md](CONFIG.md) | Переменные окружения и конфигурация |
| [API.md](API.md) | REST API документация (75+ эндпоинтов) |
| [AUTH.md](AUTH.md) | Аутентификация: JWT, TOTP, LDAP, OIDC |
| [DOCKER_DEMO.md](DOCKER_DEMO.md) | Docker демонстрация |
| [PLAYBOOK_API.md](PLAYBOOK_API.md) | Playbook API (Ansible/Terraform) |

---

## 🛠 Технологический стек

| Компонент | Технология |
|---|---|
| Backend | Rust, Axum 0.8, SQLx 0.8, Tokio 1 |
| Frontend | Vanilla JS (без фреймворков), Roboto font |
| Дизайн | Material Design (teal `#005057` sidebar) |
| Базы данных | SQLite (dev/demo) / PostgreSQL / MySQL |
| Аутентификация | JWT, bcrypt, TOTP, LDAP, OIDC |
| Шифрование | AES-256-GCM (ключи доступа) |
| CI | GitHub Actions (build + clippy + test) |

---

## ✨ Возможности (feature parity с Go-оригиналом)

- ✅ **Управление проектами** — мультипроектная архитектура с ролевой моделью
- ✅ **Шаблоны задач** — Ansible / Terraform / OpenTofu / Shell, Views/Tabs
- ✅ **Task Runner** — реальный запуск с WebSocket live-логами
- ✅ **Инвентари** — статические, динамические, Terraform workspace, файловые
- ✅ **Репозитории** — git checkout по ветке/тегу/коммиту
- ✅ **Расписания** — cron планировщик с визуальным редактором
- ✅ **Webhooks + Integration Matchers** — фильтрация входящих событий
- ✅ **Backup / Restore** — полный экспорт/импорт проекта в JSON
- ✅ **Auth** — JWT, bcrypt, TOTP (2FA + recovery codes), LDAP, OIDC
- ✅ **Secret Storages** — Vault/DVLS интеграция
- ✅ **Custom Roles** — permissions bitmask
- ✅ **Runners** — self-registration, heartbeat, per-project runner tags
- ✅ **Analytics** — статистика задач с Chart.js
- ✅ **Audit Log** — полный лог действий
- ✅ **Playbooks** — CRUD + история запусков
- ✅ **Apps** — управление типами исполнителей (ansible/terraform/bash/tofu)
- ✅ **Шифрование ключей** — AES-256-GCM + маскировка в API

---

## 📊 Статус миграции

| Компонент | Статус |
|---|---|
| Backend API (75+ эндпоинтов) | ✅ 100% |
| Тесты | ✅ 667 passed |
| Frontend (28+ страниц) | ✅ 100% |
| Аутентификация | ✅ 100% |
| Task Runner | ✅ 100% |
| Scheduler (cron) | ✅ 100% |
| Docker (demo + prod) | ✅ 100% |
| PostgreSQL схема | ✅ 100% |
| MySQL схема | ✅ 100% |

---

## 🔗 Репозитории

| | URL |
|---|---|
| Этот проект | https://github.com/tnl-o/velum |
| Go-оригинал (эталон) | https://github.com/velum/velum |
| Upstream (alexandervashurin) | https://github.com/alexandervashurin/semaphore |

---

## 📝 Лицензия

MIT © [Alexander Vashurin](https://github.com/alexandervashurin)

Оригинальный проект [Velum](https://github.com/velum/velum) на Go — используется как эталон feature parity.
