# ⚡ Быстрый старт Semaphore UI

## 🐳 Запуск через Docker (рекомендуется)

### Одна команда

```bash
./start.sh
```

### Доступ к системе

- **URL**: http://localhost:3000
- **Логин**: `admin`
- **Пароль**: `admin123`

### Управление

```bash
./start.sh              # Запуск
./start.sh --build      # Запуск с пересборкой
./start.sh --clean      # Полный сброс + запуск
./start.sh --logs       # Просмотр логов
./stop.sh               # Остановка
./stop.sh --clean       # Остановка + очистка данных
```

---

## 📦 Что внутри

| Компонент | Технология | Описание |
|-----------|------------|----------|
| **Frontend** | Vue 2 + Vuetify 2 | Все ресурсы локальные |
| **Backend** | Rust (Axum) | Высокая производительность |
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

### Установка Docker

```bash
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
# Перелогиньтесь
```

---

## 📚 Документация

- [DOCKER_RUN.md](DOCKER_RUN.md) - полный гайд по Docker-запуску
- [README.md](README.md) - основная документация
- [DOCKER_BUILD.md](web/DOCKER_BUILD.md) - сборка frontend

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
