# 📦 Публикация релиза v2.1.0

## ✅ Выполнено

### 1. Git тег создан

```bash
git tag -a v2.1.0 -m "Velum v2.1.0"
git push origin main --tags
```

**Статус:** ✅ Тег v2.1.0 запушен на GitHub

### 2. CHANGELOG обновлён

Добавлена секция [2.1.0] - 2026-03-20 в `CHANGELOG.md`

**Статус:** ✅ Закоммичено и запушено

### 3. RELEASE файл создан

Создан `RELEASE_v2.1.0.md` с полным описанием релиза

**Статус:** ✅ Закоммичено и запушено

---

## 🔔 Что нужно сделать вручную

### GitHub Releases

Поскольку gh CLI недоступен, создайте релиз вручную:

1. Перейдите на https://github.com/tnl-o/semarust/releases/new
2. Tag version: `v2.1.0`
3. Release title: `🦀 Velum v2.1.0 - Playbook Run API`
4. Описание: скопируйте из `RELEASE_v2.1.0.md`
5. Нажмите "Publish release"

**Содержимое для описания релиза:**

```markdown
# 🦀 Velum (Rust) v2.1.0

**Дата релиза:** 20 марта 2026 г.

## 🎉 Основные возможности

### Playbook Run API

Полноценный API для запуска playbook (Ansible, Terraform, Shell):

- `POST /api/project/{id}/playbooks/{playbook_id}/run` — запуск playbook
- `GET /api/project/{id}/playbook-runs` — список запусков
- `GET /api/project/{id}/playbook-runs/{id}` — запуск по ID
- `GET /api/project/{id}/playbooks/{id}/runs/stats` — статистика

### Скрипт запуска сервера

```bash
./start-server.sh start      # Запуск PostgreSQL + сервера
./start-server.sh stop       # Остановка
./start-server.sh restart    # Перезапуск
./start-server.sh status     # Статус
./start-server.sh logs       # Просмотр логов
```

### Миграция базы данных

- 29 новых колонок в таблицах `template` и `task`
- Таблица `view` для представлений шаблонов
- Поддержка всех параметров Ansible

## 📦 Установка

### Docker

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

## 🔧 Миграция

```bash
./scripts/apply-db-migration.sh
```

## 📊 Статистика

- 7 файлов добавлено
- 930 строк кода
- 29 колонок БД
- 670 тестов пройдено

[Полный changelog](https://github.com/tnl-o/semarust/blob/main/CHANGELOG.md)
```

---

## 📋 Чеклист публикации

- [x] Изменения закоммичены
- [x] Тег создан и запушен
- [x] CHANGELOG обновлён
- [x] RELEASE файл создан
- [ ] GitHub Release создан (вручную)
- [ ] Docker образ опубликован (CI/CD)
- [ ] Документация обновлена

---

## 🔗 Ссылки

- **Репозиторий:** https://github.com/tnl-o/semarust
- **Release на GitHub:** https://github.com/tnl-o/semarust/releases/tag/v2.1.0
- **Changelog:** https://github.com/tnl-o/semarust/blob/main/CHANGELOG.md
- **Документация:** https://github.com/tnl-o/semarust/tree/main/docs

---

*Дата создания релиза: 2026-03-20*
