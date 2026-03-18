# Playbook API - Итоговая документация

## Обзор

Реализован полный цикл управления Playbook в Velum на Rust - аналог Ansible Tower/AWX.

**Версия:** 0.4.3  
**Дата:** 2026-03-12  
**Статус:** ✅ Реализовано

---

## 📋 Реализованная функциональность

### 1. CRUD API для Playbook

| Endpoint | Метод | Описание |
|----------|-------|----------|
| `/api/project/{id}/playbooks` | GET | Список playbook проекта |
| `/api/project/{id}/playbooks` | POST | Создать playbook |
| `/api/project/{id}/playbooks/{id}` | GET | Получить playbook по ID |
| `/api/project/{id}/playbooks/{id}` | PUT | Обновить playbook |
| `/api/project/{id}/playbooks/{id}` | DELETE | Удалить playbook |

**Модель данных:**
```json
{
  "id": 1,
  "project_id": 1,
  "name": "deploy.yml",
  "content": "- hosts: all\n  tasks:\n    - debug:\n        msg: Hello",
  "description": "Deployment playbook",
  "playbook_type": "ansible",
  "repository_id": 5,
  "created": "2026-03-12T10:00:00Z",
  "updated": "2026-03-12T10:00:00Z"
}
```

**Файлы:**
- `rust/src/api/handlers/playbook.rs` - CRUD handlers
- `rust/src/db/sql/managers/playbook.rs` - DB менеджер
- `rust/src/models/playbook.rs` - Модели данных

---

### 2. Валидация Playbook

**Типы валидации:**

#### Ansible Playbook
- ✅ Проверка YAML синтаксиса
- ✅ Playbook должен быть списком (sequence)
- ✅ Каждый play должен содержать поле `hosts`
- ✅ Проверка структуры tasks и roles

#### Terraform Config
- ✅ Проверка YAML/HCL синтаксиса
- ✅ Проверка допустимых ключей (resource, variable, output...)

#### Shell Script
- ✅ Проверка на пустоту
- ⚠️ Рекомендация наличия shebang

**Ограничения:**
- Максимальный размер: 10 MB
- Поддерживаемые типы: `ansible`, `terraform`, `shell`

**Пример ошибки валидации:**
```json
{
  "error": "Ошибка валидации: Отсутствует обязательное поле: Play #1: hosts"
}
```

**Файлы:**
- `rust/src/validators/playbook_validator.rs` - Валидатор
- `rust/src/validators/mod.rs` - Модуль валидации

**Тесты:** 8 тестов (100% покрытие)

---

### 3. Синхронизация из Git

| Endpoint | Метод | Описание |
|----------|-------|----------|
| `/api/project/{id}/playbooks/{id}/sync` | POST | Синхронизировать из Git |
| `/api/project/{id}/playbooks/{id}/preview` | GET | Предпросмотр из Git |

**Алгоритм синхронизации:**
1. Получение playbook из БД
2. Проверка наличия `repository_id`
3. Получение repository из БД
4. Клонирование репозитория (git2)
5. Поиск файла playbook (5 вариантов путей)
6. Чтение и валидация содержимого
7. Обновление БД

**Автоматическое определение пути:**
1. `playbook_name` (например, "deploy.yml")
2. `playbook_name.yml`
3. `playbook_name.yaml`
4. `playbooks/playbook_name`
5. `playbooks/playbook_name.yml`

**Файлы:**
- `rust/src/services/playbook_sync_service.rs` - Сервис синхронизации
- `rust/src/api/handlers/playbook.rs` - Sync/Preview handlers

**Тесты:** 1 тест

---

### 4. Запуск Playbook

| Endpoint | Метод | Описание |
|----------|-------|----------|
| `/api/project/{id}/playbooks/{id}/run` | POST | Запустить playbook |

**Параметры запуска:**
```json
{
  "inventory_id": 1,
  "environment_id": 2,
  "extra_vars": {"app": "myapp", "version": "1.0"},
  "limit": "localhost",
  "tags": ["deploy", "web"],
  "skip_tags": ["debug"]
}
```

**Ответ:**
```json
{
  "task_id": 123,
  "template_id": 45,
  "status": "waiting",
  "message": "Задача создана и ожидает выполнения"
}
```

**Алгоритм запуска:**
1. Валидация запроса
2. Получение playbook из БД
3. Проверка inventory/environment (если указаны)
4. Создание/поиск template для playbook
5. Создание задачи (Task)
6. Возврат task_id

**Файлы:**
- `rust/src/services/playbook_run_service.rs` - Сервис запуска
- `rust/src/models/playbook_run.rs` - Модели запуска
- `rust/src/api/handlers/playbook.rs` - Run handler

**Тесты:** 2 теста

---

## 🛠 Технические детали

### Зависимости

```toml
[dependencies]
serde_yaml = "0.9"      # YAML парсинг
git2 = "0.20"           # Git операции
tempfile = "3"          # Временные файлы
```

### Модули

```
rust/src/
├── api/
│   ├── handlers/
│   │   └── playbook.rs         # CRUD + Sync + Preview + Run
│   └── routes.rs               # API routes
├── db/
│   ├── sql/
│   │   └── managers/
│   │       └── playbook.rs     # DB менеджер
│   └── mock.rs                 # Mock для тестов
├── models/
│   ├── playbook.rs             # CRUD модели
│   └── playbook_run.rs         # Run модели
├── services/
│   ├── playbook_sync_service.rs    # Синхронизация
│   └── playbook_run_service.rs     # Запуск
└── validators/
    └── playbook_validator.rs       # Валидация
```

### Тесты

```bash
# Запуск всех тестов
cargo test --lib

# Тесты валидации
cargo test playbook_validator

# Тесты синхронизации
cargo test playbook_sync_service

# Тесты запуска
cargo test playbook_run_service
```

**Результат:** 650 тестов пройдено ✅

---

## 📚 Примеры использования

### 1. Создание Playbook

```bash
curl -X POST http://localhost:3000/api/project/1/playbooks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "deploy.yml",
    "content": "- hosts: all\n  tasks:\n    - debug:\n        msg: Hello",
    "playbook_type": "ansible",
    "repository_id": 5
  }'
```

### 2. Синхронизация из Git

```bash
# Предпросмотр
curl -X GET http://localhost:3000/api/project/1/playbooks/1/preview \
  -H "Authorization: Bearer $TOKEN"

# Синхронизация
curl -X POST http://localhost:3000/api/project/1/playbooks/1/sync \
  -H "Authorization: Bearer $TOKEN"
```

### 3. Запуск Playbook

```bash
curl -X POST http://localhost:3000/api/project/1/playbooks/1/run \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "inventory_id": 1,
    "extra_vars": {"app": "myapp"},
    "limit": "localhost",
    "tags": ["deploy"]
  }'
```

### 4. Валидация (автоматическая)

При создании/обновлении:
- ✅ Валидный playbook → 200/201
- ❌ Невалидный → 400 Bad Request

```json
{
  "error": "Ошибка валидации: Playbook должен быть списком plays"
}
```

---

## 📊 Статистика реализации

| Метрика | Значение |
|---------|----------|
| Коммитов | 8 |
| Файлов создано | 8 |
| Файлов изменено | 45+ |
| Строк добавлено | ~3000 |
| Тестов пройдено | 650 |
| Ошибок компиляции | 0 (было 800+) |

---

## 🗺 Roadmap

### Реализовано (v0.4.1 - v0.4.3)
- ✅ CRUD API
- ✅ Валидация YAML/Ansible/Terraform/Shell
- ✅ Синхронизация из Git
- ✅ Запуск через Task
- ✅ Auto-generated templates

### В планах (v0.5.0+)
- [ ] Frontend интеграция (редактор YAML)
- [ ] SSH аутентификация для Git
- [ ] История запусков playbook
- [ ] Периодическая синхронизация
- [ ] Webhook при изменении в Git
- [ ] Интеграция с ansible-lint

---

## 🔗 Ссылки

### Документация
- [PLAYBOOK_API.md](PLAYBOOK_API.md) - CRUD API
- [PLAYBOOK_VALIDATION.md](PLAYBOOK_VALIDATION.md) - Валидация
- [PLAYBOOK_SYNC.md](PLAYBOOK_SYNC.md) - Синхронизация
- [PLAYBOOK_ROADMAP.md](PLAYBOOK_ROADMAP.md) - План развития

### API Документация
- [API.md](API.md#-playbook-api) - Общая API документация

### Код
- `rust/src/api/handlers/playbook.rs` - Handlers
- `rust/src/services/` - Сервисы
- `rust/src/validators/` - Валидация

---

## 📝 Changelog

### [0.4.3] - 2026-03-12
- ✅ Запуск Playbook через API
- ✅ Интеграция с TaskManager
- ✅ Auto-generated templates

### [0.4.2] - 2026-03-12
- ✅ Синхронизация из Git
- ✅ Preview содержимого
- ✅ git2 интеграция

### [0.4.1] - 2026-03-12
- ✅ Валидация playbook
- ✅ Ansible/Terraform/Shell
- ✅ 8 unit тестов

---

**Авторы:** Alexander Vashurin  
**Лицензия:** MIT  
**Проект:** [Velum на Rust](https://github.com/alexandervashurin/semaphore)
