# Конфигурация

> Все параметры конфигурации Velum
>
> 📖 См. также: [Быстрый старт](./quick-start.md), [Первый проект](./first-project.md), [Docker](../deployment/docker-deployment.md), [Аутентификация и безопасность](../architecture/auth-security.md)

---

## Переменные окружения

### Обязательные

| Переменная | Описание | Пример |
|------------|----------|--------|
| `VELUM_DB_URL` | Строка подключения к PostgreSQL | `postgres://user:pass@host:5432/velum` |
| `VELUM_DB_DIALECT` | Диалект базы данных | `postgres` |
| `VELUM_JWT_SECRET` | Секрет подписи JWT (мин. 32 символа) | `your-secret-key-32-bytes-long!!` |
| `VELUM_WEB_PATH` | Путь к файлам веб-интерфейса | `/app/web/public` |

### Администратор (первый запуск)

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `VELUM_ADMIN` | Имя администратора | `admin` |
| `VELUM_ADMIN_PASSWORD` | Пароль администратора | `admin123` |
| `VELUM_ADMIN_NAME` | Отображаемое имя | `Administrator` |
| `VELUM_ADMIN_EMAIL` | Электронная почта | `admin@velum.local` |

### Необязательные

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `VELUM_LDAP_*` | Настройки LDAP | — |
| `VELUM_OIDC_*` | Провайдеры OIDC | — |
| `RUST_LOG` | Уровень логирования | `info` |
| `VELUM_HA_REDIS_HOST` | Хост Redis для HA | — |
| `VELUM_HA_REDIS_PORT` | Порт Redis для HA | — |

---

## Конфигурация Docker

См. [Docker](../deployment/docker-deployment.md) для всех вариантов compose.

---

## Дополнительно

### Очередь задач через Redis

```bash
VELUM_REDIS_URL=redis://localhost:6379
```

### Telegram-бот

```bash
VELUM_TELEGRAM_TOKEN=ваш-токен-бота
VELUM_TELEGRAM_CHAT_ID=-1001234567890
```

См. также: [Telegram-бот](../resources/telegram-bot.md)

### Логирование

```bash
RUST_LOG=velum=debug,tower_http=debug
```

См. также: [Режим отладки](../troubleshooting/debug-mode.md)

---

## Следующие шаги

- [Первый проект](./first-project.md) — создайте свой первый проект
- [Аутентификация и безопасность](../architecture/auth-security.md) — настройка LDAP, OIDC, TOTP
- [Продакшен](../deployment/production-setup.md) — подготовка к продакшену
