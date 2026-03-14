# 📊 Статус БД для Playbook

## ✅ Таблицы реализованы

### 1. Таблица `playbook`

**Файлы миграций:**
- `db/postgres/002_playbooks.sql` - отдельная миграция
- `db/postgres/init-demo.sql` - включена в основную схему ✅
- `scripts/init-postgres-full.sql` - включена в полную схему ✅

**Структура таблицы:**
```sql
CREATE TABLE playbook (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,              -- YAML содержимое
    description TEXT,
    playbook_type VARCHAR(50) NOT NULL DEFAULT 'ansible',
    repository_id INTEGER REFERENCES repository(id) ON DELETE SET NULL,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

**Индексы:**
- `idx_playbook_project` - по project_id
- `idx_playbook_name` - по name
- `idx_playbook_type` - по playbook_type

**Rust модель:** `rust/src/models/playbook.rs`
- `Playbook` - основная модель
- `PlaybookCreate` - для создания
- `PlaybookUpdate` - для обновления

---

### 2. Таблица `playbook_run`

**Файлы миграций:**
- `db/postgres/003_playbook_runs.sql` - отдельная миграция
- `db/postgres/init-demo.sql` - включена в основную схему ✅
- `scripts/init-postgres-full.sql` - включена в полную схему ✅

**Структура таблицы:**
```sql
CREATE TABLE playbook_run (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    playbook_id INTEGER NOT NULL REFERENCES playbook(id) ON DELETE CASCADE,
    task_id INTEGER REFERENCES task(id) ON DELETE SET NULL,
    template_id INTEGER REFERENCES template(id) ON DELETE SET NULL,
    
    -- Статус выполнения
    status VARCHAR(50) NOT NULL DEFAULT 'waiting',
    
    -- Параметры запуска
    inventory_id INTEGER REFERENCES inventory(id) ON DELETE SET NULL,
    environment_id INTEGER REFERENCES environment(id) ON DELETE SET NULL,
    extra_vars TEXT,
    limit_hosts VARCHAR(500),
    tags TEXT,
    skip_tags TEXT,
    
    -- Результаты
    start_time TIMESTAMP WITH TIME ZONE,
    end_time TIMESTAMP WITH TIME ZONE,
    duration_seconds INTEGER,
    
    -- Статистика
    hosts_total INTEGER DEFAULT 0,
    hosts_changed INTEGER DEFAULT 0,
    hosts_unreachable INTEGER DEFAULT 0,
    hosts_failed INTEGER DEFAULT 0,
    
    -- Вывод
    output TEXT,
    error_message TEXT,
    
    -- Пользователь
    user_id INTEGER REFERENCES "user"(id) ON DELETE SET NULL,
    
    -- Метаданные
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

**Индексы:**
- `idx_playbook_run_project` - по project_id
- `idx_playbook_run_playbook` - по playbook_id
- `idx_playbook_run_task` - по task_id
- `idx_playbook_run_status` - по status
- `idx_playbook_run_created` - по created
- `idx_playbook_run_user` - по user_id

**Rust модель:** `rust/src/models/playbook_run_history.rs`
- `PlaybookRun` - основная модель
- `PlaybookRunStatus` - enum статусов (waiting, running, success, failed, cancelled)
- `PlaybookRunCreate` - для создания
- `PlaybookRunUpdate` - для обновления
- `PlaybookRunStats` - статистика
- `PlaybookRunFilter` - фильтр для поиска

**Rust запросы:** `PlaybookRunRequest`, `PlaybookRunResult` - для API

---

## 📋 Связи с другими таблицами

```
playbook
├── project_id → project(id)
└── repository_id → repository(id)

playbook_run
├── project_id → project(id)
├── playbook_id → playbook(id)
├── task_id → task(id)
├── template_id → template(id)
├── inventory_id → inventory(id)
├── environment_id → environment(id)
└── user_id → user(id)
```

---

## ✅ API Endpoints реализованы

### Playbook CRUD

| Метод | Endpoint | Описание |
|-------|----------|----------|
| GET | `/api/project/{id}/playbooks` | Список playbook |
| POST | `/api/project/{id}/playbooks` | Создать playbook |
| GET | `/api/project/{id}/playbooks/{id}` | Получить playbook |
| PUT | `/api/project/{id}/playbooks/{id}` | Обновить playbook |
| DELETE | `/api/project/{id}/playbooks/{id}` | Удалить playbook |
| POST | `/api/project/{id}/playbooks/{id}/sync` | Синхронизировать из Git |
| GET | `/api/project/{id}/playbooks/{id}/preview` | Предпросмотр из Git |
| POST | `/api/project/{id}/playbooks/{id}/run` | Запустить playbook |

### Playbook Run (история запусков)

| Метод | Endpoint | Описание |
|-------|----------|----------|
| GET | `/api/project/{id}/playbook-runs` | Список запусков |
| GET | `/api/project/{id}/playbook-runs/{id}` | Детали запуска |
| GET | `/api/project/{id}/playbook-runs/{id}/output` | Вывод запуска |

---

## 📝 Сервисы реализованы

| Сервис | Файл | Описание |
|--------|------|----------|
| `PlaybookSyncService` | `rust/src/services/playbook_sync_service.rs` | Синхронизация из Git |
| `PlaybookRunService` | `rust/src/services/playbook_run_service.rs` | Запуск playbook |
| `PlaybookRunStatusService` | `rust/src/services/playbook_run_status_service.rs` | Обновление статуса |

---

## 🎯 Frontend Vanilla JS

### Компоненты

| Компонент | Файл | Статус |
|-----------|------|--------|
| PlaybookList | `web/vanilla/js/components/playbook-list.js` | ✅ Создан |
| PlaybookForm | `web/vanilla/js/components/playbook-form.js` | ✅ Создан |

### API клиент

Методы в `web/vanilla/js/api.js`:
- `getPlaybooks(projectId)`
- `getPlaybook(projectId, playbookId)`
- `createPlaybook(projectId, data)`
- `updatePlaybook(projectId, playbookId, data)`
- `deletePlaybook(projectId, playbookId)`
- `syncPlaybook(projectId, playbookId)`
- `previewPlaybook(projectId, playbookId)`
- `runPlaybook(projectId, playbookId, data)`

### Маршруты

- `/project/:projectId/playbooks` - список playbook'ов ✅

---

## 🔍 Проверка целостности

### Внешние ключи

Все внешние ключи настроены корректно:
- ✅ `ON DELETE CASCADE` для project_id
- ✅ `ON DELETE CASCADE` для playbook_id
- ✅ `ON DELETE SET NULL` для опциональных полей

### Индексы

Все необходимые индексы созданы:
- ✅ Индексы для поиска по project_id
- ✅ Индексы для поиска по status
- ✅ Индексы для сортировки по created

---

## 📊 Демонстрационные данные

**Примечание:** Демонстрационные данные для playbook таблиц пока не добавлены в `init-demo.sql`.

**Требуется добавить:**
- Несколько тестовых playbook (Ansible, Terraform, Shell)
- Историю запусков для демонстрации

---

## ✅ Итоговый статус

| Компонент | Статус |
|-----------|--------|
| **БД таблицы** | ✅ Готово |
| **Миграции** | ✅ Готово |
| **Rust модели** | ✅ Готово |
| **API handlers** | ✅ Готово |
| **Сервисы** | ✅ Готово |
| **Frontend Vanilla JS** | ✅ Готово |
| **Frontend Vue.js** | ✅ Готово |
| **Демо данные** | ⏳ Требуется добавить |

---

*Последнее обновление: 13 марта 2026 г.*
