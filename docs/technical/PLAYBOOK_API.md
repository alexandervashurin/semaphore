# Playbook API - Реализация

## Обзор

Реализован полный CRUD API для управления Playbook (Ansible, Terraform, Shell) в Velum.

## Изменения

### Backend (Rust)

#### Новые файлы
- `rust/src/api/handlers/playbook.rs` - HTTP handlers для CRUD операций
- `rust/src/db/sql/managers/playbook.rs` - Реализация PlaybookManager для SqlStore
- `test-playbook-api.sh` - Скрипт тестирования API

#### Измененные файлы
- `rust/src/api/routes.rs` - Добавлены endpoints для Playbook API
- `rust/src/api/store_wrapper.rs` - Реализация PlaybookManager для StoreWrapper
- `rust/src/db/store.rs` - Добавлен трейт PlaybookManager
- `rust/src/db/sql/managers/mod.rs` - Добавлен модуль playbook
- `rust/src/db/sql/mod.rs` - Исправлены методы get_*_pool()
- `rust/src/db/sql/managers/*.rs` - Исправлены импорты и ошибки компиляции

### База данных

Миграция уже существует:
- `db/postgres/002_playbooks.sql` - Таблица `playbook`

### Модель данных

`rust/src/models/playbook.rs`:
- `Playbook` - основная модель
- `PlaybookCreate` - для создания
- `PlaybookUpdate` - для обновления

### API Endpoints

| Метод | URL | Описание |
|-------|-----|----------|
| GET | `/api/project/{project_id}/playbooks` | Получить список playbooks |
| POST | `/api/project/{project_id}/playbooks` | Создать playbook |
| GET | `/api/project/{project_id}/playbooks/{id}` | Получить playbook по ID |
| PUT | `/api/project/{project_id}/playbooks/{id}` | Обновить playbook |
| DELETE | `/api/project/{project_id}/playbooks/{id}` | Удалить playbook |

### Примеры использования

#### Создание Playbook

```bash
curl -X POST http://localhost:3000/api/project/1/playbooks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Deploy App",
    "content": "- hosts: all\n  tasks:\n    - name: Deploy\n      debug:\n        msg: \"Deploying...\"",
    "description": "Deployment playbook",
    "playbook_type": "ansible",
    "repository_id": null
  }'
```

#### Получение списка Playbooks

```bash
curl -X GET http://localhost:3000/api/project/1/playbooks \
  -H "Authorization: Bearer $TOKEN"
```

#### Обновление Playbook

```bash
curl -X PUT http://localhost:3000/api/project/1/playbooks/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Deploy App Updated",
    "content": "- hosts: all\n  tasks:\n    - name: Updated\n      debug:\n        msg: \"Updated!\"",
    "description": "Updated deployment playbook",
    "playbook_type": "ansible"
  }'
```

#### Удаление Playbook

```bash
curl -X DELETE http://localhost:3000/api/project/1/playbooks/1 \
  -H "Authorization: Bearer $TOKEN"
```

## Тестирование

### Автоматическое тестирование

```bash
# Настроить переменные окружения
export BASE_URL=http://localhost:3000/api
export PROJECT_ID=1
export TOKEN=your_token_here

# Запустить тесты
./test-playbook-api.sh
```

### Ручное тестирование

```bash
# Получить токен
TOKEN=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"login":"admin","password":"admin123"}' \
  | jq -r '.token')

# Создать playbook
curl -X POST http://localhost:3000/api/project/1/playbooks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Test Playbook",
    "content": "- hosts: localhost\n  tasks:\n    - debug:\n        msg: Hello",
    "playbook_type": "ansible"
  }' | jq .
```

## Компиляция

Проект успешно компилируется:

```bash
cd rust
cargo check
# Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Поддерживаемые типы Playbook

- `ansible` - Playbook Ansible (YAML)
- `terraform` - Конфигурация Terraform (HCL)
- `shell` - Shell скрипты

## Интеграция с Repository

Playbook может быть связан с Git репозиторием через поле `repository_id`. Это позволяет:
- Загружать playbook из Git
- Синхронизировать изменения
- Использовать версии из Git

## Frontend

Frontend страница для управления Playbook уже существует:
- Страница Playbooks в проекте
- CRUD операции через inventory API
- Отображение списка playbook

## Документация

- [API.md](API.md#-playbook-api) - Полная документация API
- [test-playbook-api.sh](test-playbook-api.sh) - Скрипт тестирования

## Статус

✅ Реализовано:
- CRUD API для Playbook
- Поддержка SQLite/PostgreSQL/MySQL
- Интеграция с StoreWrapper
- HTTP handlers и routes
- Тесты API
- Документация

⚠️ Требуется доработка:
- Валидация YAML контента
- Интеграция с Git repository
- Запуск playbook через template
- Просмотр истории запусков
- Интеграция с frontend (Vue.js)

## Авторы

- Backend: Rust implementation
- Frontend: Vue.js (существующая страница)
- Documentation: Full API docs

## Дата реализации

2026-03-12
