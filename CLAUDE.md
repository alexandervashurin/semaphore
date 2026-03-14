# CLAUDE.md — Инструкции для AI-агента

> Этот файл читается Claude Code при каждом запуске. Следуй порядку действий строго.

---

## Конечная цель проекта

**Полная миграция Semaphore UI с Go на Rust** — feature parity с Go-оригиналом, опубликованная на GitHub.

| Репозиторий | URL |
|---|---|
| Наш (origin) | https://github.com/tnl-o/rust_semaphore |
| Upstream (alexandervashurin) | https://github.com/alexandervashurin/semaphore |
| Go-оригинал (эталон фич) | https://github.com/semaphoreui/semaphore |

Ориентируйся на **Go-оригинал** как источник правды о том, что должно работать.
Ориентируйся на **MASTER_PLAN.md** как живой план задач.

---

## Порядок действий при каждом запуске

### 1. Проверка состояния git

```bash
git status
git log --oneline -5
```

- Если есть незакоммиченные изменения — сначала разберись с ними

---

### 2. Fetch + Merge из upstream

```bash
git fetch upstream
git log upstream/main --oneline -5
git merge upstream/main --no-edit
```

- Remote `upstream` = `https://github.com/alexandervashurin/semaphore`
- При конфликтах: разрешай сохраняя наши изменения, затем `cargo check`
- После успешного merge — сразу пуш в origin:
  ```bash
  git push origin main
  ```

---

### 3. Проверка MASTER_PLAN.md

Читай секции **"Текущее состояние"** и **"Известные проблемы и блокеры"**.

- Код — источник правды. Если план расходится с кодом — обнови план
- Выбирай следующую задачу по приоритету: 🔴 → 🟠 → 🟡
- Смотри на Go-оригинал чтобы понять что ещё нужно реализовать

---

### 4. Работа: реализация задач

1. Создай todo-список через `TodoWrite`
2. Читай файлы перед редактированием
3. Помечай задачи `in_progress` → `completed` в реальном времени
4. `cargo check` после каждого значимого изменения

**Приоритет задач из MASTER_PLAN:**
- Фаза 6 — Фронтенд Vanilla JS миграция (в процессе, главный блокер)
- Фаза 8 — Prod-готовность (E2E тесты, Docker multi-stage)
- Auth refresh token, LDAP — закрыты
- Оставшиеся открытые задачи из таблицы блокеров

---

### 5. Коммиты — после каждой завершённой задачи

Формат **Conventional Commits**:

```
feat(auth): add refresh token endpoint
fix(db): handle null values in migration
docs(plan): update MASTER_PLAN — close B-06b
test(runner): add ansible execution tests
```

Стейджинг конкретными файлами (не `git add -A`):
```bash
git add rust/src/api/handlers/auth.rs MASTER_PLAN.md
git commit -m "feat(auth): ..."
```

---

### 6. Пуш на GitHub — после каждого коммита

```bash
git push origin main
```

Пушить сразу после коммита — это рабочий процесс проекта.
Не накапливать локальные коммиты без пуша.

---

### 7. Обновление MASTER_PLAN.md — обязательно

После каждой реализованной задачи:

1. Обновить статус в таблице "Текущее состояние": `⬜` → `✅`
2. Закрыть блокер в таблице "Известные проблемы": добавить `✅ Закрыт`
3. Обновить дату: `**Последнее обновление:** YYYY-MM-DD`
4. Коммитить и пушить вместе с кодом или отдельно

---

## Технический стек

| Компонент | Технология |
|---|---|
| Бэкенд | Rust, Axum 0.8, SQLx 0.8, Tokio 1 |
| БД | SQLite (dev) / PostgreSQL (prod) / MySQL |
| Auth | JWT (jsonwebtoken), bcrypt, TOTP, LDAP, OIDC |
| Фронтенд | Vanilla JS (миграция с Vue 2) |
| CI | `.github/workflows/rust.yml` (build + clippy + test) |

---

## Команды разработки

```bash
# Проверка компиляции
cd rust && cargo check

# Линтер (должен быть 0 warnings)
cd rust && cargo clippy -- -D warnings

# Тесты
cd rust && cargo test

# Запуск локально (SQLite)
cd rust && SEMAPHORE_DB_PATH=/tmp/semaphore.db cargo run -- server

# Запуск всего через Docker
docker compose up -d
```

---

## Правила

- Читай файл перед редактированием — всегда
- Не создавай новые файлы если можно отредактировать существующий
- Не добавляй комментарии и docstrings к коду который не менял
- Пуш делать после каждого коммита (это норма для этого проекта)
- При merge-конфликтах: HEAD (наш код) имеет приоритет если сомневаешься
- Смотри на Go-оригинал (`semaphoreui/semaphore`) как эталон API и поведения
