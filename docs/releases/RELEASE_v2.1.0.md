# 🦀 Velum (Rust) v2.1.0

**Дата релиза:** 20 марта 2026 г.

## 🎉 Основные возможности

### Playbook Run API

Полноценный API для запуска playbook (Ansible, Terraform, Shell):

```bash
# Запуск playbook
POST /api/project/{id}/playbooks/{playbook_id}/run

# Список запусков
GET /api/project/{id}/playbook-runs

# Запуск по ID
GET /api/project/{id}/playbook-runs/{id}

# Статистика
GET /api/project/{id}/playbooks/{id}/runs/stats
```

### Скрипт запуска сервера

Новый удобный скрипт для управления сервером:

```bash
./start-server.sh start      # Запуск PostgreSQL + сервера
./start-server.sh stop       # Остановка
./start-server.sh restart    # Перезапуск
./start-server.sh status     # Статус
./start-server.sh logs       # Просмотр логов
```

### Миграция базы данных

Автоматическое добавление 29 колонок в таблицы `template` и `task`:

- Поддержка всех параметров запуска Ansible (--limit, --tags, --skip-tags)
- Связь с playbook, inventory, environment, repository
- Поддержка view (представления шаблонов)
- Поля для CI/CD интеграции

## 📦 Установка

### DEB пакет (Debian/Ubuntu)

```bash
# Скачать
wget https://github.com/tnl-o/semarust/releases/download/v2.1.0/velum-2.1.0.deb

# Установить
sudo dpkg -i velum-2.1.0.deb
sudo apt install -f

# Создать admin
sudo velum user add --username admin --email admin@example.com \
  --password admin123 --admin

# Запустить
sudo systemctl start velum
sudo systemctl enable velum
```

### Docker (рекомендуется)

```bash
docker run -d \
  --name semaphore \
  -p 3000:3000 \
  -e SEMAPHORE_DB_DIALECT=sqlite \
  -e SEMAPHORE_ADMIN=admin \
  -e SEMAPHORE_ADMIN_PASSWORD=admin123 \
  -v semaphore_data:/var/lib/semaphore \
  ghcr.io/tnl-o/semarust:v2.1.0
```

### Docker Compose (с PostgreSQL)

```bash
docker compose up -d
```

### Ручная установка

```bash
# Скачать бинарник
curl -L https://github.com/tnl-o/semarust/releases/download/v2.1.0/semaphore-linux-amd64 \
  -o /usr/local/bin/semaphore
chmod +x /usr/local/bin/semaphore

# Запустить
semaphore server
```

## 🔧 Миграция с v2.0.0

### Автоматическая миграция

Миграции применяются автоматически при запуске сервера.

### Ручная миграция

```bash
# Применить миграцию БД
./scripts/apply-db-migration.sh

# Или вручную
docker exec -i semaphore-db psql -U semaphore -d semaphore \
  < db/postgres/migrations/003_full_schema_update.sql
```

## 🧪 Тестирование

### Проверка сборки

```bash
cd rust
cargo build --release
cargo test
cargo clippy -- -D warnings
```

### Проверка Playbook Run API

```bash
# Получить токен
TOKEN=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"demo123"}' | jq -r '.token')

# Запустить playbook
curl -X POST http://localhost:3000/api/project/1/playbooks/2/run \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{}'

# Проверить статус
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/project/1/playbook-runs/1
```

## 📊 Статистика

| Метрика | Значение |
|---------|----------|
| **Файлов добавлено** | 7 |
| **Строк добавлено** | 930 |
| **Колонок БД добавлено** | 29 |
| **Таблиц БД обновлено** | 3 |
| **Тестов пройдено** | 670 |

## 🐛 Исправления

- Добавлены все недостающие колонки для поддержки Playbook Run API
- Создана таблица `view` для представлений шаблонов
- Исправлены ошибки компиляции (0 warnings)
- Обновлена документация по развёртыванию

## 📚 Документация

- [START_SERVER.md](START_SERVER.md) — запуск сервера
- [db/postgres/MIGRATION_003.md](db/postgres/MIGRATION_003.md) — миграция БД
- [db/postgres/PLAYBOOK_DB_FIX.md](db/postgres/PLAYBOOK_DB_FIX.md) — исправления БД
- [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) — руководство по развёртыванию
- [CHANGELOG.md](CHANGELOG.md) — история изменений

## 🔗 Ссылки

- **GitHub:** https://github.com/tnl-o/semarust
- **Docker:** https://github.com/tnl-o/semarust/pkgs/container/semarust
- **Документация:** https://github.com/tnl-o/semarust/tree/main/docs

## 🙏 Благодарности

- @tnl-o за разработку
- @claude за помощь в разработке
- Всем контрибьюторам

---

**Полный список изменений:** https://github.com/tnl-o/semarust/compare/v2.0.0...v2.1.0
