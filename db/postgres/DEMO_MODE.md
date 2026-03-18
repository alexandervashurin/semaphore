# 🚀 Демо-режим Velum

Полное руководство по запуску и использованию демо-режима Velum с демонстрационными данными.

## 📋 Оглавление

- [Быстрый старт](#быстрый-старт)
- [Что входит в демо-данные](#что-входит-в-демо-данные)
- [Режимы запуска](#режимы-запуска)
- [Учётные записи](#учётные-записи)
- [Команды управления](#команды-управления)

---

## ⚡ Быстрый старт

```bash
# Инициализация демо-режима
./scripts/start-demo-mode.sh

# Запуск backend и frontend
./start.sh hybrid

# Открыть в браузере
# http://localhost:3000
```

---

## 📊 Что входит в демо-данные

| Сущность | Количество | Описание |
|----------|------------|----------|
| 👥 Пользователи | 4 | Admin, John Doe, Jane Smith, DevOps |
| 📁 Проекты | 4 | Infrastructure, Web App, Database, Security |
| 🔐 Ключи доступа | 5 | SSH ключи и login/password |
| 📝 Инвентари | 5 | Production, Staging, Clusters |
| 🗂️ Репозитории | 5 | Playbooks для всех проектов |
| ⚙️ Окружения | 5 | Переменные для всех сред |
| 📋 Шаблоны | 12 | Задачи для всех проектов |
| ⏰ Расписания | 4 | Автоматические задачи |
| 🎯 Задачи | 6 | Выполненные и активные |
| 📊 События | 6 | История операций |

---

## 👤 Учётные записи

**Пароль для всех пользователей:** `demo123`

| Логин | Имя | Роль | Доступ |
|-------|-----|------|--------|
| `admin` | Administrator | Администратор | Все проекты |
| `john.doe` | John Doe | Менеджер | Web Application |
| `jane.smith` | Jane Smith | Менеджер | Database Management |
| `devops` | DevOps Engineer | Исполнитель | Все проекты |

---

## 📁 Проекты

### 1. Demo Infrastructure
- Шаблоны: Deploy Infrastructure, Update Servers, Staging Deploy
- Инвентари: Production Servers, Staging Environment

### 2. Web Application Deployment
- Шаблоны: Deploy Web App, Rollback Web App, Scale Web App
- Инвентари: Web App Cluster

### 3. Database Management
- Шаблоны: Backup Databases, Restore Database, DB Health Check
- Инвентари: Database Cluster

### 4. Security & Compliance
- Шаблоны: Security Scan, Compliance Check, Patch Security
- Инвентари: Security Scan Targets

---

## 🛠️ Команды управления

```bash
# Инициализация демо-режима
./scripts/start-demo-mode.sh

# Проверка статуса
./scripts/start-demo-mode.sh --status

# Просмотр логов
./scripts/start-demo-mode.sh --logs

# Остановка
./scripts/start-demo-mode.sh --stop

# Полная переустановка
./scripts/start-demo-mode.sh --clean
```

---

## 🔍 Проверка данных

```bash
# Статус БД
docker exec semaphore-db psql -U semaphore -d semaphore -c \
  "SELECT 'Users' as entity, COUNT(*) as count FROM \"user\"
   UNION ALL SELECT 'Projects', COUNT(*) FROM project
   UNION ALL SELECT 'Templates', COUNT(*) FROM template
   UNION ALL SELECT 'Tasks', COUNT(*) FROM task;"
```

---

## 📝 Дополнительные ресурсы

- [API документация](../../API.md)
- [CRUD демонстрация](../../CRUD_DEMO.md)
- [Настройка окружения](../../SETUP_ENV.md)

---

**Версия демо-данных:** 2.0  
**Последнее обновление:** Март 2026
