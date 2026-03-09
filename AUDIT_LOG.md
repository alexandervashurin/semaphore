# 🔍 Audit Log API - Документация

> **Расширенное логирование действий пользователей для аудита и безопасности**

---

## 📋 Содержание

1. [Обзор](#обзор)
2. [Модель данных](#модель-данных)
3. [API Endpoints](#api-endpoints)
4. [Примеры использования](#примеры-использования)
5. [Фильтрация и поиск](#фильтрация-и-поиск)

---

## 📖 Обзор

Audit Log предоставляет детальное логирование всех значимых действий в системе:

- **Аутентификация**: вход, выход, неудачные попытки
- **Управление пользователями**: создание, обновление, удаление
- **Управление проектами**: все операции с проектами
- **Задачи**: создание, запуск, завершение, ошибки
- **Шаблоны**: CRUD операции, запуски
- **Инфраструктура**: инвентари, репозитории, окружения, ключи
- **Системные события**: конфигурация, бэкапы, миграции

---

## 🗃️ Модель данных

### AuditLog

| Поле | Тип | Описание |
|------|-----|----------|
| `id` | i64 | Уникальный ID записи |
| `project_id` | Option<i64> | ID проекта (если применимо) |
| `user_id` | Option<i64> | ID пользователя |
| `username` | Option<String> | Имя пользователя |
| `action` | AuditAction | Тип действия |
| `object_type` | AuditObjectType | Тип объекта |
| `object_id` | Option<i64> | ID объекта |
| `object_name` | Option<String> | Название объекта |
| `description` | String | Описание действия |
| `level` | AuditLevel | Уровень важности |
| `ip_address` | Option<String> | IP адрес |
| `user_agent` | Option<String> | User agent |
| `details` | Option<Json> | Дополнительные данные |
| `created` | DateTime<Utc> | Время создания |

### AuditAction

```rust
// Аутентификация
Login, Logout, LoginFailed, PasswordChanged, TwoFactorEnabled
// Пользователи
UserCreated, UserUpdated, UserDeleted, UserJoinedProject
// Проекты
ProjectCreated, ProjectUpdated, ProjectDeleted
// Задачи
TaskCreated, TaskStarted, TaskCompleted, TaskFailed, TaskStopped
// Шаблоны
TemplateCreated, TemplateUpdated, TemplateDeleted, TemplateRun
// Инфраструктура
InventoryCreated, RepositoryCreated, EnvironmentCreated, AccessKeyCreated
// Интеграции
IntegrationCreated, WebhookTriggered
// Расписания
ScheduleCreated, ScheduleTriggered
// Раннеры
RunnerCreated, RunnerConnected, RunnerDisconnected
// Системные
ConfigChanged, BackupCreated, MigrationApplied
```

### AuditLevel

- `info` - Информационное
- `warning` - Предупреждение
- `error` - Ошибка
- `critical` - Критическое

---

## 🌐 API Endpoints

### GET /api/audit-log

Поиск записей audit log с фильтрацией и пагинацией.

**Требуемые права**: Администратор

**Параметры запроса**:

| Параметр | Тип | Описание |
|----------|-----|----------|
| `project_id` | i64 | Фильтр по проекту |
| `user_id` | i64 | Фильтр по пользователю |
| `username` | string | Поиск по имени пользователя |
| `action` | string | Фильтр по действию |
| `object_type` | string | Фильтр по типу объекта |
| `object_id` | i64 | Фильтр по ID объекта |
| `level` | string | Фильтр по уровню |
| `search` | string | Поиск по описанию |
| `date_from` | DateTime | Начало периода |
| `date_to` | DateTime | Окончание периода |
| `limit` | i64 | Лимит записей (по умолчанию 50) |
| `offset` | i64 | Смещение |
| `sort` | string | Поле сортировки |
| `order` | string | Порядок (asc/desc) |

**Ответ**:

```json
{
  "total": 100,
  "records": [...],
  "limit": 50,
  "offset": 0
}
```

---

### GET /api/audit-log/:id

Получение записи audit log по ID.

**Требуемые права**: Администратор

**Ответ**:

```json
{
  "id": 1,
  "project_id": 1,
  "user_id": 1,
  "username": "admin",
  "action": "login",
  "object_type": "user",
  "description": "Пользователь admin выполнил вход",
  "level": "info",
  "ip_address": "192.168.1.1",
  "created": "2026-03-09T10:00:00Z"
}
```

---

### GET /api/project/:project_id/audit-log

Получение audit log проекта.

**Требуемые права**: Доступ к проекту

**Параметры**:

| Параметр | Тип | Описание |
|----------|-----|----------|
| `limit` | i64 | Лимит записей |
| `offset` | i64 | Смещение |

---

### DELETE /api/audit-log/clear

Очистка всего audit log.

**Требуемые права**: Супер-администратор

**Ответ**:

```json
{
  "deleted": 1000,
  "message": "Audit log очищен"
}
```

---

### DELETE /api/audit-log/expiry

Удаление старых записей.

**Требуемые права**: Администратор

**Параметры**:

| Параметр | Тип | Описание |
|----------|-----|----------|
| `before` | DateTime | Удалить записи до даты |

**Ответ**:

```json
{
  "deleted": 500,
  "before": "2026-01-01T00:00:00Z",
  "message": "Удалено 500 записей до 2026-01-01T00:00:00Z"
}
```

---

## 🔍 Примеры использования

### Поиск всех входов в систему

```bash
curl -X GET "http://localhost:3000/api/audit-log?action=login" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Поиск действий конкретного пользователя

```bash
curl -X GET "http://localhost:3000/api/audit-log?user_id=1&limit=100" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Поиск записей за период

```bash
curl -X GET "http://localhost:3000/api/audit-log?date_from=2026-03-01T00:00:00Z&date_to=2026-03-31T23:59:59Z" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Поиск неудачных попыток входа

```bash
curl -X GET "http://localhost:3000/api/audit-log?action=login_failed&level=warning" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Поиск по описанию

```bash
curl -X GET "http://localhost:3000/api/audit-log?search=template" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

## 🎛️ Фильтрация и поиск

### Комбинирование фильтров

```bash
# Все действия пользователя admin в проекте 1
curl -X GET "http://localhost:3000/api/audit-log?user_id=1&project_id=1" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Все ошибки при выполнении задач
curl -X GET "http://localhost:3000/api/audit-log?action=task_failed&level=error" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Действия с шаблонами за последнюю неделю
curl -X GET "http://localhost:3000/api/audit-log?object_type=template&date_from=2026-03-02T00:00:00Z" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Сортировка

```bash
# Последние записи сначала
curl -X GET "http://localhost:3000/api/audit-log?sort=created&order=desc" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Сортировка по пользователю
curl -X GET "http://localhost:3000/api/audit-log?sort=user_id&order=asc" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Пагинация

```bash
# Вторая страница по 50 записей
curl -X GET "http://localhost:3000/api/audit-log?limit=50&offset=50" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

## 🗄️ База данных

### Миграция

Файл: `db/postgres/001_audit_log.sql`

```sql
CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    user_id BIGINT,
    username VARCHAR(255),
    action VARCHAR(100) NOT NULL,
    object_type VARCHAR(50) NOT NULL,
    object_id BIGINT,
    object_name VARCHAR(255),
    description TEXT NOT NULL,
    level VARCHAR(20) NOT NULL DEFAULT 'info',
    ip_address VARCHAR(45),
    user_agent TEXT,
    details JSONB,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### Индексы

- `idx_audit_log_project_id` - по проекту
- `idx_audit_log_user_id` - по пользователю
- `idx_audit_log_action` - по действию
- `idx_audit_log_object_type` - по типу объекта
- `idx_audit_log_level` - по уровню
- `idx_audit_log_created` - по времени
- `idx_audit_log_details_gin` - GIN индекс для JSONB

---

## 📝 Реализация

### Файлы

| Файл | Описание |
|------|----------|
| `src/models/audit_log.rs` | Модели данных |
| `src/db/sql/audit_log.rs` | SQL репозиторий |
| `src/api/handlers/audit_log.rs` | API handlers |
| `src/db/store.rs` | AuditLogManager trait |
| `db/postgres/001_audit_log.sql` | Миграция БД |

### Добавление записи audit log

```rust
use crate::models::audit_log::{AuditAction, AuditObjectType, AuditLevel};

state.store.create_audit_log(
    Some(project_id),
    Some(user_id),
    Some(username.clone()),
    &AuditAction::TaskCreated,
    &AuditObjectType::Task,
    Some(task_id),
    Some(task_name.clone()),
    format!("Создана задача {}", task_name),
    &AuditLevel::Info,
    Some(ip_address),
    Some(user_agent),
    Some(json!({"task_type": "ansible"})),
).await?;
```

---

*Последнее обновление: 9 марта 2026 г.*
