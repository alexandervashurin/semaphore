# 💾 Backup & Restore

> Руководство по резервному копированию и восстановлению Semaphore UI
> **Последнее обновление:** 10 марта 2026 г.

---

## 📋 Содержание

1. [Быстрый старт](#быстрый-старт)
2. [Backup](#backup)
3. [Restore](#restore)
4. [Автоматизация](#автоматизация)
5. [Переменные окружения](#переменные-окружения)

---

## 🚀 Быстрый старт

### Создание бэкапа

```bash
# Быстрый бэкап
./scripts/backup.sh

# С переменными окружения
SEMAPHORE_DB_TYPE=postgres \
SEMAPHORE_DB_HOST=localhost \
SEMAPHORE_DB_NAME=semaphore \
./scripts/backup.sh
```

### Восстановление из бэкапа

```bash
# Восстановление из последнего бэкапа
./scripts/restore.sh backups/semaphore_backup_20260310_120000.tar.gz
```

---

## 💾 Backup

### Что включает бэкап

1. **База данных**
   - PostgreSQL: дамп через `pg_dump`
   - MySQL: дамп через `mysqldump`
   - SQLite: копия файла

2. **Конфигурация**
   - `.env.example`
   - `docker-compose.yml`
   - `nginx.conf`
   - `db/postgres/`
   - `deployment/`

### Использование

```bash
# Базовый запуск
./scripts/backup.sh

# С кастомной директорией для бэкапов
SEMAPHORE_BACKUP_DIR=/mnt/backups ./scripts/backup.sh

# С хранением бэкапов 30 дней
SEMAPHORE_BACKUP_RETENTION_DAYS=30 ./scripts/backup.sh
```

### Вывод

```
============================================================================
              Semaphore UI - Backup Script
============================================================================

[INFO] Проверка зависимостей...
[OK] Все зависимости найдены
[INFO] Создание директории для бэкапов: /path/to/backups
[INFO] Бэкап базы данных (postgres)...
[OK] PostgreSQL бэкап создан: /tmp/db_backup_20260310_120000.sql
[INFO] Бэкап конфигурации...
[OK] Конфигурация скопирована
[INFO] Создание архива...
[OK] Архив создан: /path/to/backups/semaphore_backup_20260310_120000.tar.gz
[INFO] Размер архива: 2.5M
[INFO] Очистка бэкапов старше 7 дней...
[OK] Осталось бэкапов: 5

============================================================================
                    Backup завершен успешно!
============================================================================

  Файл бэкапа: /path/to/backups/semaphore_backup_20260310_120000.tar.gz
  Размер: 2.5M
  Дата: 2026-03-10 12:00:00

  Для восстановления используйте:
    ./scripts/restore.sh /path/to/backups/semaphore_backup_20260310_120000.tar.gz

============================================================================
```

---

## 🔄 Restore

### ⚠️ Предупреждение

**Восстановление заменит текущие данные в базе данных!**

### Использование

```bash
# Восстановление из файла
./scripts/restore.sh backups/semaphore_backup_20260310_120000.tar.gz

# С подтверждением (по умолчанию)
# Скрипт запросит подтверждение перед началом
```

### Процесс восстановления

1. Проверка зависимостей
2. Распаковка архива
3. Очистка текущей базы данных
4. Восстановление из дампа
5. Восстановление конфигурации
6. Очистка временных файлов

### Вывод

```
============================================================================
              Semaphore UI - Restore Script
============================================================================

[WARN] ВНИМАНИЕ: Восстановление заменит текущие данные!

  Файл бэкапа: backups/semaphore_backup_20260310_120000.tar.gz
  Тип БД: postgres
  База данных: semaphore

Продолжить? (yes/no): yes

[INFO] Проверка зависимостей...
[OK] Все зависимости найдены
[INFO] Распаковка архива...
[OK] Архив распакован
[INFO] Восстановление базы данных (postgres)...
[INFO] Очистка существующей базы...
[OK] PostgreSQL восстановлена
[INFO] Восстановление конфигурации...
[OK] Конфигурация восстановлена
[INFO] Очистка временных файлов...
[OK] Временные файлы удалены

============================================================================
                    Восстановление завершено!
============================================================================

  База данных: semaphore (postgres)
  Дата бэкапа: 2026-03-10

  Для запуска Semaphore выполните:
    ./demo-start.sh

============================================================================
```

---

## ⏰ Автоматизация

### Cron job для ежедневного бэкапа

```bash
# Редактировать crontab
crontab -e

# Добавить задачу (ежедневно в 2:00)
0 2 * * * /path/to/semaphore/scripts/backup.sh >> /var/log/semaphore-backup.log 2>&1
```

### Systemd timer

**/etc/systemd/system/semaphore-backup.service:**
```ini
[Unit]
Description=Semaphore Backup
After=postgresql.service

[Service]
Type=oneshot
User=semaphore
Environment=SEMAPHORE_DB_TYPE=postgres
Environment=SEMAPHORE_DB_HOST=localhost
Environment=SEMAPHORE_DB_NAME=semaphore
ExecStart=/path/to/semaphore/scripts/backup.sh
```

**/etc/systemd/system/semaphore-backup.timer:**
```ini
[Unit]
Description=Run Semaphore Backup Daily

[Timer]
OnCalendar=*-*-* 02:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

**Активация:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable semaphore-backup.timer
sudo systemctl start semaphore-backup.timer
```

---

## 🔧 Переменные окружения

| Переменная | По умолчанию | Описание |
|------------|--------------|----------|
| `SEMAPHORE_BACKUP_DIR` | `./backups` | Директория для хранения бэкапов |
| `SEMAPHORE_BACKUP_RETENTION_DAYS` | `7` | Количество дней хранения бэкапов |
| `SEMAPHORE_DB_TYPE` | `postgres` | Тип БД: postgres, mysql, sqlite |
| `SEMAPHORE_DB_HOST` | `localhost` | Хост базы данных |
| `SEMAPHORE_DB_PORT` | `5432` | Порт базы данных |
| `SEMAPHORE_DB_NAME` | `semaphore` | Имя базы данных |
| `SEMAPHORE_DB_USER` | `semaphore` | Пользователь базы данных |
| `SEMAPHORE_DB_PASS` | - | Пароль базы данных |
| `SEMAPHORE_SQLITE_PATH` | `/tmp/semaphore.db` | Путь к SQLite файлу |

---

## 📊 Примеры

### PostgreSQL

```bash
# Бэкап
SEMAPHORE_DB_TYPE=postgres \
SEMAPHORE_DB_HOST=db.example.com \
SEMAPHORE_DB_NAME=semaphore \
SEMAPHORE_DB_USER=semaphore \
SEMAPHORE_DB_PASS=secret \
./scripts/backup.sh

# Восстановление
SEMAPHORE_DB_TYPE=postgres \
SEMAPHORE_DB_HOST=db.example.com \
SEMAPHORE_DB_NAME=semaphore \
SEMAPHORE_DB_USER=semaphore \
SEMAPHORE_DB_PASS=secret \
./scripts/restore.sh backups/semaphore_backup_*.tar.gz
```

### MySQL

```bash
# Бэкап
SEMAPHORE_DB_TYPE=mysql \
SEMAPHORE_DB_HOST=db.example.com \
SEMAPHORE_DB_PORT=3306 \
SEMAPHORE_DB_NAME=semaphore \
SEMAPHORE_DB_USER=semaphore \
SEMAPHORE_DB_PASS=secret \
./scripts/backup.sh

# Восстановление
SEMAPHORE_DB_TYPE=mysql \
./scripts/restore.sh backups/semaphore_backup_*.tar.gz
```

### SQLite

```bash
# Бэкап
SEMAPHORE_DB_TYPE=sqlite \
SEMAPHORE_SQLITE_PATH=/var/lib/semaphore/semaphore.db \
./scripts/backup.sh

# Восстановление
SEMAPHORE_DB_TYPE=sqlite \
SEMAPHORE_SQLITE_PATH=/var/lib/semaphore/semaphore.db \
./scripts/restore.sh backups/semaphore_backup_*.tar.gz
```

---

## 🔍 Troubleshooting

### Ошибка: "Зависимость не найдена: pg_dump"

```bash
# PostgreSQL
sudo apt-get install postgresql-client

# MySQL
sudo apt-get install mysql-client
```

### Ошибка: "Permission denied"

```bash
# Проверить права на скрипты
chmod +x scripts/backup.sh scripts/restore.sh

# Проверить права на директорию бэкапов
chmod 755 backups/
```

### Ошибка: "Database does not exist"

```bash
# Создать базу данных
createdb -h localhost -U semaphore semaphore

# Или для MySQL
mysql -h localhost -u semaphore -p -e "CREATE DATABASE semaphore;"
```

---

## 📚 Дополнительные ресурсы

- [PostgreSQL Backup](https://www.postgresql.org/docs/current/backup.html)
- [MySQL Backup](https://dev.mysql.com/doc/refman/8.0/en/backup.html)
- [SQLite Backup](https://www.sqlite.org/backup.html)

---

*Документ будет обновляться по мере добавления новых функций*
