# 🔧 Настройка окружения

## Автоматическая настройка

### Быстрый старт

```bash
# Инициализация и настройка (выберите режим)
./semaphore.sh init native    # SQLite
./semaphore.sh init hybrid    # PostgreSQL в Docker
```

Команда `init` автоматически:
1. ✅ Создаст `.env` файл с нужными переменными
2. ✅ Применит миграции БД
3. ✅ Создаст пользователя admin
4. ✅ Настроит все необходимые параметры

### Примеры использования

#### 1. Настройка SQLite (для тестирования)

```bash
./semaphore.sh init native
./semaphore.sh start native
```

**Результат:**
```bash
cat .env

SEMAPHORE_DB_DIALECT=sqlite
SEMAPHORE_DB_PATH=/tmp/semaphore.db
SEMAPHORE_WEB_PATH=./web/public
SEMAPHORE_TMP_PATH=/tmp/semaphore
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
RUST_LOG=info
```

**Учётные данные:**
- `admin` / `admin123`

#### 2. Настройка PostgreSQL (для продакшена)

```bash
./semaphore.sh init hybrid
./semaphore.sh start hybrid
```

**Результат:**
```bash
cat .env

SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_URL=postgres://semaphore:semaphore_pass@localhost:5432/semaphore
SEMAPHORE_WEB_PATH=./web/public
SEMAPHORE_TMP_PATH=/tmp/semaphore
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
RUST_LOG=info
```

**Учётные данные:**
- `admin` / `admin123`

#### 3. Настройка PostgreSQL с демо-данными

```bash
# Запуск PostgreSQL с демо-данными
docker-compose -f docker-compose.postgres.yml up -d

# Инициализация
./semaphore.sh init hybrid
./semaphore.sh start hybrid
```

**Учётные данные (демо):**
- `admin` / `demo123`
- `john.doe` / `demo123`
- `jane.smith` / `demo123`
- `devops` / `demo123`

---

## Ручная настройка

### Вариант 1: SQLite (по умолчанию)

```bash
# Создаём .env файл
cat > .env <<EOF
SEMAPHORE_DB_DIALECT=sqlite
SEMAPHORE_DB_PATH=/tmp/semaphore.db
SEMAPHORE_DB_URL=sqlite:///tmp/semaphore.db
SEMAPHORE_JWT_SECRET=$(openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | head -c 64)
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
SEMAPHORE_WEB_PATH=./web/public
RUST_LOG=info
SEMAPHORE_TMP_PATH=/tmp/semaphore
EOF
```

### Вариант 2: PostgreSQL

```bash
# Создаём .env файл
cat > .env <<EOF
SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_URL=postgres://semaphore:semaphore_pass@localhost:5432/semaphore
SEMAPHORE_JWT_SECRET=$(openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | head -c 64)
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
SEMAPHORE_WEB_PATH=./web/public
RUST_LOG=info
SEMAPHORE_TMP_PATH=/tmp/semaphore
EOF
```

### Вариант 3: Демо-окружение

```bash
# 1. Запускаем Docker с демо-данными
./demo-start.sh

# 2. Создаём .env файл
cat > .env <<EOF
SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_URL=postgres://semaphore:semaphore_pass@localhost:5432/semaphore
SEMAPHORE_JWT_SECRET=$(openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | head -c 64)
SEMAPHORE_DEMO_MODE=true
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
SEMAPHORE_WEB_PATH=./web/public
RUST_LOG=info
SEMAPHORE_TMP_PATH=/tmp/semaphore
EOF
```

---

## Переменные окружения

### Обязательные

| Переменная | Описание | Пример |
|------------|----------|--------|
| `SEMAPHORE_DB_DIALECT` | Тип БД | `sqlite`, `postgres`, `mysql` |
| `SEMAPHORE_DB_URL` | URL подключения к БД | `postgres://user:pass@host:port/db` |
| `SEMAPHORE_JWT_SECRET` | Секрет для JWT токенов | Любая строка 32+ символов |

### Рекомендуемые

| Переменная | Описание | По умолчанию |
|------------|----------|------------|
| `SEMAPHORE_TCP_ADDRESS` | Адрес для прослушивания | `0.0.0.0:3000` |
| `SEMAPHORE_WEB_PATH` | Путь к web файлам | `./web/public` |
| `RUST_LOG` | Уровень логирования | `info` |
| `SEMAPHORE_TMP_PATH` | Временная директория | `/tmp/semaphore` |

### Опциональные

| Переменная | Описание | Пример |
|------------|----------|--------|
| `SEMAPHORE_DEMO_MODE` | Включить демо режим | `true` |
| `SEMAPHORE_HA_ENABLE` | HA режим | `false` |
| `SEMAPHORE_LDAP_ENABLE` | LDAP аутентификация | `false` |
| `SEMAPHORE_AUTH_TOTP_ENABLE` | TOTP (2FA) | `false` |

---

## Проверка конфигурации

### 1. Проверка .env файла

```bash
# Проверка наличия файла
ls -la .env

# Просмотр содержимого
cat .env
```

### 2. Проверка подключения к БД

```bash
# Для SQLite
sqlite3 /tmp/semaphore.db ".tables"

# Для PostgreSQL
docker-compose exec -T db psql -U semaphore -d semaphore -c "\dt"
```

### 3. Тестовый запуск

```bash
# Запуск сервера
cargo run -- server --host 0.0.0.0 --port 3000

# Проверка API
curl http://localhost:3000/api
```

---

## Решение проблем

### Проблема: .env файл не читается

**Решение:**
```bash
# Проверка формата файла
file .env

# Должно быть: ASCII text
# Если другой формат - пересоздайте файл

# Проверка кодировки
cat -A .env

# Не должно быть ^M символов (Windows line endings)
# Если есть - конвертируйте:
dos2unix .env
```

### Проблема: JWT_SECRET не установлен

**Решение:**
```bash
# Сгенерируйте новый секрет
export SEMAPHORE_JWT_SECRET=$(openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | head -c 64)

# Добавьте в .env
echo "SEMAPHORE_JWT_SECRET=$SEMAPHORE_JWT_SECRET" >> .env
```

### Проблема: PostgreSQL не подключается

**Решение:**
```bash
# Проверьте, что PostgreSQL запущен
docker-compose ps

# Проверьте URL подключения
echo $SEMAPHORE_DB_URL

# Должно быть:
# postgres://semaphore:semaphore_pass@localhost:5432/semaphore

# Проверьте подключение вручную
psql postgres://semaphore:semaphore_pass@localhost:5432/semaphore
```

---

## Безопасность

### ⚠️ Важно для продакшена

1. **Измените JWT_SECRET**
   ```bash
   # Сгенерируйте случайную строку
   openssl rand -base64 64
   ```

2. **Измените пароль БД**
   ```bash
   # Не используйте demo123 или semaphore_pass
   ```

3. **Ограничьте доступ к .env**
   ```bash
   chmod 600 .env
   ```

4. **Не коммитьте .env в Git**
   ```bash
   # .env уже в .gitignore
   # Но проверьте:
   git check-ignore .env
   ```

5. **Используйте secrets manager**
   - Docker Secrets
   - AWS Secrets Manager
   - HashiCorp Vault

---

## Скрипты

### setup-env.sh

Интерактивный скрипт для настройки `.env`:

```bash
# Запуск
./setup-env.sh

# Опции:
# - Автоматическая генерация JWT_SECRET
# - Выбор типа БД (SQLite/PostgreSQL/Демо)
# - Настройка порта и уровня логирования
# - Создание резервной копии существующего файла
```

### demo-start.sh

Скрипт для запуска демо-окружения:

```bash
# Запуск Docker (PostgreSQL + Frontend)
./demo-start.sh

# Запуск Backend
./demo-start.sh --backend

# Остановка
./demo-start.sh --stop

# Сброс данных
./demo-start.sh --reset
```

---

## Дополнительные ресурсы

- [CONFIG.md](CONFIG.md) - Полная документация по конфигурации
- [CRUD_DEMO.md](CRUD_DEMO.md) - Руководство по CRUD демо
- [ЗАПУСК_ДЕМО.md](ЗАПУСК_ДЕМО.md) - Инструкция по запуску

---

## Примеры .env файлов

### Для разработки (SQLite)

```bash
# .env.development
SEMAPHORE_DB_DIALECT=sqlite
SEMAPHORE_DB_PATH=/tmp/semaphore_dev.db
SEMAPHORE_JWT_SECRET=dev_secret_not_for_production
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
SEMAPHORE_WEB_PATH=./web/public
RUST_LOG=debug
SEMAPHORE_TMP_PATH=/tmp/semaphore
```

### Для продакшена (PostgreSQL)

```bash
# .env.production
SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_URL=postgres://semaphore:STRONG_PASSWORD@db.example.com:5432/semaphore
SEMAPHORE_JWT_SECRET=ОЧЕНЬ_ДЛИННЫЙ_СЕКРЕТ_ДЛЯ_ПРОДАКШЕНА
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
SEMAPHORE_WEB_PATH=./web/public
RUST_LOG=warn
SEMAPHORE_TMP_PATH=/var/tmp/semaphore
```

### Для тестирования

```bash
# .env.test
SEMAPHORE_DB_DIALECT=sqlite
SEMAPHORE_DB_PATH=/tmp/semaphore_test.db
SEMAPHORE_JWT_SECRET=test_secret
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3001
SEMAPHORE_WEB_PATH=./web/public
RUST_LOG=error
SEMAPHORE_TMP_PATH=/tmp/semaphore_test
```

---

**Готово!** 🎉 Теперь ваше окружение настроено и готово к работе!
