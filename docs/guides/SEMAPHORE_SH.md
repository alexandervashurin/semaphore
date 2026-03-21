# 🛠️ Velum - Универсальный скрипт управления

## 📖 Обзор

Скрипт `semaphore.sh` предоставляет единый интерфейс для управления всеми аспектами Velum:
- Запуск в различных режимах (Native, Hybrid, Docker)
- Остановка и перезапуск сервисов
- Очистка данных
- Инициализация БД
- Просмотр статуса и логов
- Сборка проекта

## 🚀 Быстрый старт

```bash
# Первый запуск (инициализация БД)
./semaphore.sh init hybrid

# Запуск сервера
./semaphore.sh start hybrid

# Проверка статуса
./semaphore.sh status
```

## 📋 Команды

### start [РЕЖИМ]

Запуск сервиса в указанном режиме.

**Режимы:**
- `native` - SQLite + Backend + Frontend на хосте (минимальные зависимости)
- `hybrid` - PostgreSQL в Docker + Backend + Frontend на хосте (рекомендуется)
- `docker` - Все сервисы в Docker

**Примеры:**
```bash
# Запуск с SQLite
./semaphore.sh start native

# Запуск с PostgreSQL в Docker (рекомендуется)
./semaphore.sh start hybrid

# Запуск всех сервисов в Docker
./semaphore.sh start docker
```

---

### stop

Остановка всех запущенных сервисов.

**Пример:**
```bash
./semaphore.sh stop
```

Автоматически определяет что запущено (native/hybrid/docker) и останавливает соответствующие сервисы.

---

### restart

Перезапуск сервисов.

**Пример:**
```bash
./semaphore.sh restart
```

---

### clean

Очистка данных (БД, volumes).

**Пример:**
```bash
./semaphore.sh clean
```

⚠️ **Внимание:** Все данные будут удалены!

---

### init [РЕЖИМ]

Инициализация базы данных:
- Применение миграций
- Создание пользователя admin

**Примеры:**
```bash
# Инициализация SQLite
./semaphore.sh init native

# Инициализация PostgreSQL
./semaphore.sh init hybrid
```

**Учётные данные после инициализации:**
- Логин: `admin`
- Пароль: `admin123`

---

### status

Показ текущего статуса сервисов:
- Статус контейнеров Docker
- Статус backend процесса
- Доступные volumes
- URL доступа

**Пример:**
```bash
./semaphore.sh status
```

**Пример вывода:**
```
╔════════════════════════════════════════════════════════╗
║ Статус Velum                                    ║
╚════════════════════════════════════════════════════════╝

Контейнеры:
  semaphore-db - Up 2 hours

Volumes:
  semaphore_postgres_data

Backend:
  ✓ Запущен (PID: 12345)

Доступ:
  http://localhost:3000
```

---

### logs

Просмотр логов в реальном времени.

**Пример:**
```bash
./semaphore.sh logs
```

Автоматически определяет режим и показывает соответствующие логи.

---

### build

Сборка проекта:
- Компиляция Rust backend
- Сборка frontend (если установлен Node.js)

**Пример:**
```bash
./semaphore.sh build
```

---

### help

Показ справки по всем командам.

**Пример:**
```bash
./semaphore.sh help
```

---

## 🎯 Режимы запуска

### Native (SQLite)

**Зависимости:**
- ✅ Rust/Cargo
- ❌ Docker (не требуется)
- ❌ Node.js (опционально для frontend)

**Команды:**
```bash
./semaphore.sh init native
./semaphore.sh start native
./semaphore.sh stop
./semaphore.sh clean
```

**Когда использовать:**
- Быстрое тестирование
- Разработка без Docker
- Минимальные зависимости

---

### Hybrid (PostgreSQL в Docker) ⭐ Рекомендуется

**Зависимости:**
- ✅ Rust/Cargo
- ✅ Docker
- ❌ Node.js (опционально для frontend)

**Команды:**
```bash
./semaphore.sh init hybrid
./semaphore.sh start hybrid
./semaphore.sh stop
./semaphore.sh clean
./semaphore.sh restart
```

**Когда использовать:**
- Продакшен окружение
- Тестирование с PostgreSQL
- Быстрая разработка с полноценной БД

---

### Docker (все сервисы в Docker)

**Зависимости:**
- ❌ Rust/Cargo (не требуется)
- ✅ Docker
- ❌ Node.js (не требуется)

**Команды:**
```bash
./semaphore.sh start docker
./semaphore.sh stop
./semaphore.sh clean
./semaphore.sh restart
```

**Когда использовать:**
- Полная изоляция окружения
- Продакшен развёртывание
- Тестирование в изолированной среде

---

## 📊 Сравнение режимов

| Характеристика | Native | Hybrid | Docker |
|----------------|--------|--------|--------|
| **Зависимости** | Rust | Rust + Docker | Docker |
| **База данных** | SQLite | PostgreSQL | PostgreSQL |
| **Backend** | На хосте | На хосте | В Docker |
| **Frontend** | На хосте | На хосте | В Docker |
| **Производительность** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Изоляция** | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Рекомендация** | Тестирование | Разработка/Продакшен | Продакшен |

---

## 🔧 Полезные команды

### Управление сервисами

```bash
# Запуск в режиме hybrid
./semaphore.sh start hybrid

# Остановка
./semaphore.sh stop

# Перезапуск
./semaphore.sh restart

# Проверка статуса
./semaphore.sh status
```

### Работа с БД

```bash
# Инициализация БД
./semaphore.sh init hybrid

# Очистка данных
./semaphore.sh clean

# Просмотр логов БД (hybrid/docker)
./semaphore.sh logs
```

### Сборка

```bash
# Сборка проекта
./semaphore.sh build

# Пересборка с очисткой
cargo clean && ./semaphore.sh build
```

---

## 🐛 Troubleshooting

### Backend не запускается

```bash
# Проверить логи
./semaphore.sh logs

# Проверить статус
./semaphore.sh status

# Пересобрать
./semaphore.sh build
```

### PostgreSQL не запускается (hybrid/docker)

```bash
# Очистить volumes и перезапустить
./semaphore.sh clean
./semaphore.sh init hybrid

# Проверить логи Docker
docker logs semaphore-db
```

### Порт 3000 занят

Измените порт в `.env` файле:
```
SEMAPHORE_TCP_ADDRESS=0.0.0.0:8080
```

---

## 📚 Дополнительная документация

- [README.md](README.md) - Основная документация
- [CONFIG.md](CONFIG.md) - Конфигурация
- [API.md](API.md) - API документация
- [ЗАПУСК_ДЕМО.md](ЗАПУСК_ДЕМО.md) - Запуск демо-режима
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Решение проблем

---

## 🎯 Миграция со старых скриптов

Если вы использовали старые скрипты, замените их:

| Старый скрипт | Новая команда |
|---------------|---------------|
| `./start.sh native` | `./semaphore.sh start native` |
| `./start.sh hybrid` | `./semaphore.sh start hybrid` |
| `./start.sh docker` | `./semaphore.sh start docker` |
| `./start.sh --stop` | `./semaphore.sh stop` |
| `./start.sh --logs` | `./semaphore.sh logs` |
| `./start.sh --clean` | `./semaphore.sh clean` |
| `./start.sh --init` | `./semaphore.sh init` |
| `./stop.sh` | `./semaphore.sh stop` |
| `./cleanup.sh` | `./semaphore.sh clean` |
| `./setup-env.sh` | (интегрировано в `init`) |

---

*Последнее обновление: 13 марта 2026 г.*
