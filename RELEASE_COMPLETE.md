# 📦 Релиз v2.1.0 опубликован!

**Дата:** 20 марта 2026 г.

## ✅ Выполнено

### 1. DEB пакет создан и загружен

```
📦 velum-2.1.0.deb
📊 Размер: 6.8 MB
🏗️ Архитектура: amd64
📍 Бинарный файл: /usr/bin/velum (12 MB)
```

**Статус:** ✅ Загружен в GitHub Release

### 2. GitHub Release опубликован

- **Тег:** v2.1.0
- **Название:** 🦀 Velum v2.1.0 - Playbook Run API
- **URL:** https://github.com/tnl-o/semarust/releases/tag/v2.1.0
- **Asset:** velum-2.1.0.deb ✅

### 3. Документация обновлена

- ✅ RELEASE_v2.1.0.md — инструкция по установке DEB
- ✅ docs/DEB_PACKAGE.md — полная документация по DEB пакету
- ✅ docs/DEPLOYMENT.md — секция DEB добавлена
- ✅ CHANGELOG.md — v2.1.0 changelog

### 4. Скрипты созданы

- ✅ scripts/build-deb.sh — сборка DEB пакета
- ✅ scripts/apply-db-migration.sh — миграции БД
- ✅ start-server.sh — запуск сервера

---

## 📊 Статистика релиза

| Метрика | Значение |
|---------|----------|
| **Коммитов** | 6 |
| **Файлов добавлено** | 9 |
| **Строк кода** | 1,674 |
| **DEB пакетов** | 1 (6.8 MB) |
| **Колонок БД** | 29 |
| **Тестов пройдено** | 670 |

---

## 🎯 Основные изменения v2.1.0

### Playbook Run API
- ✅ POST /api/project/{id}/playbooks/{playbook_id}/run
- ✅ GET /api/project/{id}/playbook-runs
- ✅ GET /api/project/{id}/playbook-runs/{id}
- ✅ GET /api/project/{id}/playbooks/{id}/runs/stats

### DEB пакет
- ✅ Автоматическая установка systemd сервиса
- ✅ Конфигурация в /etc/velum/
- ✅ Данные в /var/lib/velum/
- ✅ Логи в /var/log/velum/
- ✅ Security ограничения

### Миграция БД
- ✅ Таблица `view` для представлений
- ✅ 11 колонок в `template`
- ✅ 18 колонок в `task`

---

## 🚀 Установка DEB пакета

```bash
# Скачать
wget https://github.com/tnl-o/semarust/releases/download/v2.1.0/velum-2.1.0.deb

# Установить
sudo dpkg -i velum-2.1.0.deb
sudo apt install -f

# Создать admin пользователя
sudo velum user add \
  --username admin \
  --email admin@example.com \
  --password admin123 \
  --admin

# Запустить сервис
sudo systemctl start velum
sudo systemctl enable velum

# Проверить статус
systemctl status velum

# Открыть в браузере
# http://localhost:3000
```

---

## 📁 Файлы релиза

| Файл | Размер | Описание |
|------|--------|----------|
| velum-2.1.0.deb | 6.8 MB | DEB пакет для Debian/Ubuntu |

---

## 🔗 Ссылки

- **GitHub Release:** https://github.com/tnl-o/semarust/releases/tag/v2.1.0
- **Changelog:** https://github.com/tnl-o/semarust/blob/main/CHANGELOG.md
- **DEB Документация:** https://github.com/tnl-o/semarust/blob/main/docs/DEB_PACKAGE.md
- **Deployment:** https://github.com/tnl-o/semarust/blob/main/docs/DEPLOYMENT.md

---

## 📝 Последние коммиты

```
fdd0cce docs: RELEASE v2.1.0 - добавлена инструкция по установке DEB пакета
2f63db8 docs: DEB пакет в DEPLOYMENT.md
277ee86 feat: DEB пакет для Velum
467ae62 docs: RELEASE v2.1.0 - Playbook Run API, миграция БД, скрипт запуска
8f81f4a feat: Playbook API запуск, миграция БД и скрипт запуска сервера
```

---

## ✅ Чеклист релиза

- [x] CHANGELOG обновлён
- [x] Git тег создан и запушен
- [x] DEB пакет собран
- [x] DEB пакет загружен в Release
- [x] RELEASE файл обновлён
- [x] Документация обновлена
- [x] Скрипты созданы
- [x] Все коммиты запушены

---

**Релиз v2.1.0 готов к использованию! 🎉**
