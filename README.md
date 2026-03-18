# 🦀 Velum на Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.80+-blue.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-655%20passed-brightgreen.svg)]()
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Migration](https://img.shields.io/badge/migration-100%25-brightgreen.svg)]()
[![Frontend](https://img.shields.io/badge/frontend-Vue%202-brightgreen.svg)]()
[![Production Ready](https://img.shields.io/badge/status-production%20ready-brightgreen.svg)]()

**Полная миграция Velum на Rust** - высокопроизводительная, безопасная и надёжная система автоматизации для Ansible, Terraform, OpenTofu, Terragrunt, PowerShell и других DevOps-инструментов.

## 🎯 Демо-режим

> **Попробуйте прямо сейчас!** Полноценное демо-окружение с готовыми данными.

```bash
# Быстрый старт демо-режима
./semaphore.sh init hybrid
./semaphore.sh start hybrid

# Откройте в браузере
http://localhost:3000
```

**Учётные данные (пароль для всех: demo123):**
- `admin` / `demo123` (администратор)
- `john.doe` / `demo123` (менеджер)
- `jane.smith` / `demo123` (менеджер)
- `devops` / `demo123` (исполнитель)

**Демо-данные:** 4 проекта, 12 шаблонов, 4 расписания, 6 задач

📖 **Подробная документация**: [db/postgres/DEMO_MODE.md](db/postgres/DEMO_MODE.md)

---

## 🚀 Быстрый Старт

### Требования

- Rust 1.80 или новее
- Cargo
- Docker (опционально, для Docker-режимов)

---

### 📋 Режимы запуска

#### Режим 1: Native (чистый запуск на хосте) ⭐ Для разработки

SQLite + Backend + Frontend на хосте. Минимальные зависимости.

```bash
# Первый запуск с инициализацией
./semaphore.sh init native

# Запуск сервера
./semaphore.sh start native
```

**Доступ:** http://localhost:3000

**Учётные данные:**
- `admin` / `admin123`

**Полезные команды:**
```bash
./semaphore.sh stop              # Остановить backend
./semaphore.sh logs              # Просмотр логов
./semaphore.sh clean             # Удалить БД
./semaphore.sh status            # Показать статус
```

---

#### Режим 2: Hybrid (PostgreSQL в Docker, остальное на хосте) ⭐ Рекомендуется для продакшена

PostgreSQL в Docker, Backend и Frontend на хосте.

```bash
# Первый запуск с инициализацией
./semaphore.sh init hybrid

# Запуск сервера
./semaphore.sh start hybrid
```

**Доступ:** http://localhost:3000

**Учётные данные (демо):**
- `admin` / `demo123`

**Полезные команды:**
```bash
./semaphore.sh stop              # Остановить сервисы
./semaphore.sh logs              # Просмотр логов
./semaphore.sh clean             # Очистить данные БД
./semaphore.sh status            # Показать статус
```

---

#### Режим 3: Docker (всё в Docker)

Frontend + PostgreSQL + Backend в Docker контейнерах.

```bash
# Запуск всех сервисов
./semaphore.sh start docker
```

**Доступ:** http://localhost

**Учётные данные (демо):**
- `admin` / `demo123`

**Полезные команды:**
```bash
./semaphore.sh stop              # Остановить сервисы
./semaphore.sh logs              # Просмотр логов
./semaphore.sh clean             # Очистить volumes
./semaphore.sh status            # Показать статус
```

---

### 🔧 Ручной запуск (для разработки)

#### Сборка frontend
```bash
./web/build.sh
```

#### Запуск backend напрямую
```bash
cd rust

# С SQLite
export SEMAPHORE_DB_DIALECT=sqlite
export SEMAPHORE_DB_PATH=/tmp/semaphore.db
cargo run -- server --host 0.0.0.0 --port 3000

# С PostgreSQL
export SEMAPHORE_DB_DIALECT=postgres
export SEMAPHORE_DB_URL=postgres://semaphore:semaphore_pass@localhost:5432/semaphore
cargo run -- server --host 0.0.0.0 --port 3000
```

#### Создание администратора
```bash
cd rust
cargo run -- user add \
  --username admin \
  --name "Administrator" \
  --email admin@localhost \
  --password admin123 \
  --admin
```
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
| [PLAYBOOK_API.md](PLAYBOOK_API.md) | 📚 Playbook API (Ansible/Terraform) |
| [scripts/README.md](scripts/README.md) | Скрипты запуска |

---

## 🛠 Технологический Стек

- **Backend:** Rust + Axum + SQLx
- **Frontend:** Vue 2
- **Базы данных:** SQLite, PostgreSQL, MySQL
- **Аутентификация:** JWT + bcrypt
- **Автоматизация:** Ansible, Terraform, OpenTofu, Terragrunt, PowerShell

## ✨ Возможности

- ✅ **Управление проектами** - мультипроектная архитектура с ролевой моделью
- ✅ **Шаблоны задач** - настройка параметров запуска для Ansible/Terraform
- ✅ **Инвентари** - динамические и статические инвентари Ansible
- ✅ **Расписания** - автоматический запуск задач по cron
- ✅ **Playbook API** - CRUD для Playbook (Ansible, Terraform, Shell) 🆕
- ✅ **Аудит логирование** - полный аудит всех действий
- ✅ **Webhooks** - интеграция с внешними системами
- ✅ **Terraform State** - управление состоянием Terraform
- ✅ **Хранилище секретов** - безопасное хранение чувствительных данных

---

## 📝 Лицензия

MIT © [Alexander Vashurin](https://github.com/alexandervashurin)

Оригинальный проект [Velum](https://github.com/velum/velum) на Go.
