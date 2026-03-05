# ⚡ Быстрый старт Semaphore UI

## 🐳 Запуск через Docker (demo)

### 1️⃣ Frontend + БД

```bash
./start.sh
```

### 2️⃣ Backend (отдельно)

```bash
./start.sh --backend
# или
cd rust && cargo run -- server
```

### 3️⃣ Доступ

- **Frontend**: http://localhost
- **Backend**: http://localhost:3000
- **Логин**: `admin` / `admin123`

---

## 🛠 Управление

```bash
./start.sh              # Запуск frontend + БД
./start.sh --backend    # + Запуск backend
./start.sh --build      # + Пересборка образов
./start.sh --clean      # + Очистка данных БД
./start.sh --logs       # Просмотр логов

./stop.sh               # Остановка
./stop.sh --clean       # Остановка + очистка данных
```

---

## 📦 Что внутри

| Компонент | Технология | Описание |
|-----------|------------|----------|
| **Frontend** | Vue 2 + Nginx | Все ресурсы локальные |
| **Backend** | Rust (Axum) | Запускается отдельно |
| **БД** | PostgreSQL 15 | Демонстрационные данные |

---

## 👥 Демо-пользователи

| Логин | Пароль | Роль |
|-------|--------|------|
| `admin` | `demo123` | Admin |
| `john.doe` | `demo123` | Manager |
| `jane.smith` | `demo123` | Developer |
| `devops` | `demo123` | DevOps |

---

## 🔧 Требования

- Docker 20.x+
- Docker Compose 2.x+
- Rust 1.75+ (для backend)

### Установка Docker

```bash
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
```

---

## 📚 Документация

- [DOCKER_DEMO.md](DOCKER_DEMO.md) — полная инструкция по Docker
- [README.md](README.md) — основная документация
- [web/DOCKER_BUILD.md](web/DOCKER_BUILD.md) — сборка frontend

---

## ❓ Проблемы?

```bash
# Проверка статуса
docker-compose ps

# Просмотр логов
docker-compose logs -f

# Перезапуск
docker-compose restart

# Полный сброс
./stop.sh --clean
./start.sh --build
```

---

**🎉 Приятной работы!**
