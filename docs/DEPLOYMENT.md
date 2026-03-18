# Deployment Guide for Velum (Rust)

**Версия:** v2.1.0  
**Дата:** 17 марта 2026 г.

---

## Содержание

1. [Варианты деплоя](#1-варианты-деплоя)
2. [Docker (рекомендуется)](#2-docker-рекомендуется)
3. [Systemd (Linux)](#3-systemd-linux)
4. [Kubernetes](#4-kubernetes)
5. [Ручная установка](#5-ручная-установка)
6. [Конфигурация](#6-конфигурация)
7. [Миграции БД](#7-миграции-бд)
8. [Backup и восстановление](#8-backup-и-восстановление)
9. [Мониторинг](#9-мониторинг)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. Варианты деплоя

| Метод | Сложность | Production | Описание |
|-------|-----------|------------|----------|
| **Docker** | 🟢 Низкая | ✅ Да | Рекомендуется для большинства случаев |
| **Docker Compose** | 🟢 Низкая | ✅ Да | Для локальной разработки и тестирования |
| **Systemd** | 🟡 Средняя | ✅ Да | Для bare-metal серверов |
| **Kubernetes** | 🔴 Высокая | ✅ Да | Для масштабирования и HA |
| **Ручная** | 🔴 Высокая | ⚠️ Ограниченно | Для отладки и разработки |

---

## 2. Docker (рекомендуется)

### Быстрый старт

```bash
docker run -d \
  --name semaphore \
  -p 3000:3000 \
  -e SEMAPHORE_DB_DIALECT=sqlite \
  -e SEMAPHORE_ADMIN=admin \
  -e SEMAPHORE_ADMIN_PASSWORD=password123 \
  -e SEMAPHORE_ADMIN_EMAIL=admin@example.com \
  -v semaphore_data:/var/lib/semaphore \
  ghcr.io/tnl-o/semarust:latest
```

### Docker Compose (с PostgreSQL)

```yaml
version: '3.8'

services:
  semaphore:
    image: ghcr.io/tnl-o/semarust:latest
    ports:
      - "3000:3000"
    environment:
      SEMAPHORE_DB_DIALECT: postgres
      SEMAPHORE_DB_HOST: db
      SEMAPHORE_DB_PORT: 5432
      SEMAPHORE_DB_NAME: semaphore
      SEMAPHORE_DB_USER: semaphore
      SEMAPHORE_DB_PASS: semaphore_password
      SEMAPHORE_ADMIN: admin
      SEMAPHORE_ADMIN_PASSWORD: password123
      SEMAPHORE_ADMIN_EMAIL: admin@example.com
    volumes:
      - semaphore_config:/etc/semaphore
    depends_on:
      db:
        condition: service_healthy
    restart: unless-stopped

  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: semaphore
      POSTGRES_USER: semaphore
      POSTGRES_PASSWORD: semaphore_password
    volumes:
      - semaphore_db:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U semaphore"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  semaphore_config:
  semaphore_db:
```

**Запуск:**
```bash
docker-compose up -d
```

**Проверка:**
```bash
docker-compose ps
curl http://localhost:3000/api/health
```

### Переменные окружения

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_DB_DIALECT` | Тип БД (sqlite, postgres, mysql) | sqlite |
| `SEMAPHORE_DB_HOST` | Хост БД | localhost |
| `SEMAPHORE_DB_PORT` | Порт БД | 5432 (postgres), 3306 (mysql) |
| `SEMAPHORE_DB_NAME` | Имя БД | semaphore |
| `SEMAPHORE_DB_USER` | Пользователь БД | semaphore |
| `SEMAPHORE_DB_PASS` | Пароль БД | semaphore |
| `SEMAPHORE_ADMIN` | Имя admin пользователя | admin |
| `SEMAPHORE_ADMIN_PASSWORD` | Пароль admin | password |
| `SEMAPHORE_ADMIN_EMAIL` | Email admin | admin@localhost |
| `SEMAPHORE_BIND` | Адрес прослушивания | :3000 |
| `SEMAPHORE_BASE_PATH` | Base path для reverse proxy | (пусто) |

---

## 3. Systemd (Linux)

### Установка

```bash
# Скачать бинарник
curl -L https://github.com/tnl-o/semarust/releases/latest/download/semaphore-linux-amd64 \
  -o /usr/local/bin/semaphore
chmod +x /usr/local/bin/semaphore

# Создать пользователя
useradd -r -s /bin/false semaphore

# Создать директорию
mkdir -p /var/lib/semaphore
chown semaphore:semaphore /var/lib/semaphore
```

### Конфигурация

Создать `/etc/semaphore/config.json`:
```json
{
  "bolt": {
    "host": "/var/lib/semaphore/database.bolt"
  },
  "dialect": "bolt",
  "addr": ":3000",
  "admin": {
    "name": "admin",
    "password": "password123",
    "email": "admin@example.com"
  }
}
```

### Service файл

Создать `/etc/systemd/system/semaphore.service`:
```ini
[Unit]
Description=Velum (Rust)
Documentation=https://github.com/tnl-o/semarust
After=network.target

[Service]
Type=simple
User=semaphore
Group=semaphore
ExecStart=/usr/local/bin/semaphore server --config /etc/semaphore/config.json
Restart=on-failure
RestartSec=5
Environment="SEMAPHORE_ADMIN=admin"
Environment="SEMAPHORE_ADMIN_PASSWORD=password123"

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/semaphore

[Install]
WantedBy=multi-user.target
```

### Запуск

```bash
systemctl daemon-reload
systemctl enable semaphore
systemctl start semaphore
systemctl status semaphore
```

---

## 4. Kubernetes

### Helm Chart

```bash
helm repo add semarust https://tnl-o.github.io/semarust-helm
helm repo update
helm install semaphore semarust/semaphore \
  --namespace semaphore \
  --create-namespace \
  --set admin.password=password123
```

### Manifest (упрощённый)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: semaphore
  namespace: semaphore
spec:
  replicas: 1
  selector:
    matchLabels:
      app: semaphore
  template:
    metadata:
      labels:
        app: semaphore
    spec:
      containers:
      - name: semaphore
        image: ghcr.io/tnl-o/semarust:latest
        ports:
        - containerPort: 3000
        env:
        - name: SEMAPHORE_DB_DIALECT
          value: "postgres"
        - name: SEMAPHORE_DB_HOST
          value: "postgres-service"
        - name: SEMAPHORE_ADMIN
          value: "admin"
        - name: SEMAPHORE_ADMIN_PASSWORD
          value: "password123"
        livenessProbe:
          httpGet:
            path: /api/health/live
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /api/health/ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: semaphore-service
  namespace: semaphore
spec:
  selector:
    app: semaphore
  ports:
  - port: 80
    targetPort: 3000
  type: ClusterIP
```

---

## 5. Ручная установка

### Требования

- Rust 1.75+
- PostgreSQL 13+ / MySQL 8+ / SQLite 3
- Ansible (опционально, для запуска playbook)

### Сборка из исходников

```bash
git clone https://github.com/tnl-o/semarust.git
cd semarust/rust
cargo build --release
cp target/release/semaphore /usr/local/bin/
```

### Запуск

```bash
# Инициализация
semaphore setup

# Запуск сервера
semaphore server
```

---

## 6. Конфигурация

### Файл конфигурации

Расположение: `/etc/semaphore/config.json`

```json
{
  "bolt": {
    "host": "/var/lib/semaphore/database.bolt"
  },
  "dialect": "bolt",
  "addr": ":3000",
  "base_path": "/semaphore",
  "admin": {
    "name": "admin",
    "password": "password123",
    "email": "admin@example.com"
  },
  "auto_backup": {
    "enabled": true,
    "interval_hours": 24,
    "backup_path": "/var/backups/semaphore",
    "max_backups": 7,
    "compress": true
  }
}
```

### Reverse Proxy (Nginx)

```nginx
server {
    listen 80;
    server_name semaphore.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Reverse Proxy (Apache)

```apache
<VirtualHost *:80>
    ServerName semaphore.example.com

    ProxyPreserveHost On
    ProxyPass / http://localhost:3000/
    ProxyPassReverse / http://localhost:3000/

    RequestHeader set X-Forwarded-Proto "http"
    RequestHeader set X-Real-IP "%{REMOTE_ADDR}s"
</VirtualHost>
```

---

## 7. Миграции БД

### Автоматические миграции

Миграции применяются автоматически при запуске:
```bash
semaphore server
```

### Ручные миграции

```bash
# Применить миграции
semaphore migrate

# Проверить статус
semaphore migrate --status
```

### Откат миграций

```bash
# Откатить последнюю миграцию
semaphore migrate --down

# Откатить к конкретной версии
semaphore migrate --down --version 20240101000000
```

---

## 8. Backup и восстановление

### Автоматический backup

Включить в конфигурации:
```json
{
  "auto_backup": {
    "enabled": true,
    "interval_hours": 24,
    "backup_path": "/var/backups/semaphore",
    "max_backups": 7,
    "compress": true
  }
}
```

### Ручной backup

```bash
# Экспорт проекта
semaphore project export --id 1 > backup.json

# Импорт проекта
semaphore project import --file backup.json
```

### Восстановление из backup

```bash
# Остановить сервис
systemctl stop semaphore

# Восстановить БД
semaphore restore --file backup.json

# Запустить сервис
systemctl start semaphore
```

---

## 9. Мониторинг

### Health Checks

```bash
# Basic health check
curl http://localhost:3000/api/health

# Liveness probe
curl http://localhost:3000/api/health/live

# Readiness probe
curl http://localhost:3000/api/health/ready
```

### Prometheus Metrics

```bash
# Метрики
curl http://localhost:3000/metrics
```

**Основные метрики:**
- `semaphore_tasks_total` — всего задач
- `semaphore_tasks_success` — успешных задач
- `semaphore_tasks_failed` — неудачных задач
- `semaphore_tasks_running` — запущенных задач

### Grafana Dashboard

Импортировать дашборд из `deployment/grafana/dashboard.json`:
```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d @deployment/grafana/dashboard.json \
  http://grafana:3000/api/dashboards/db
```

---

## 10. Troubleshooting

### Проблема: Сервис не запускается

**Проверка логов:**
```bash
journalctl -u semaphore -f
```

**Проверка портов:**
```bash
netstat -tlnp | grep 3000
```

### Проблема: Ошибки подключения к БД

**Проверка подключения:**
```bash
# PostgreSQL
psql -h localhost -U semaphore -d semaphore -c "SELECT 1"

# MySQL
mysql -h localhost -u semaphore -p semaphore -e "SELECT 1"
```

### Проблема: Миграции не применяются

**Ручное применение:**
```bash
semaphore migrate --verbose
```

**Сброс миграций (ОПАСНО!):**
```bash
semaphore migrate --reset
```

### Проблема: Высокое потребление памяти

**Ограничение памяти (Docker):**
```yaml
services:
  semaphore:
    deploy:
      resources:
        limits:
          memory: 512M
```

**Ограничение памяти (Systemd):**
```ini
[Service]
MemoryLimit=512M
```

---

## Приложения

### A. Security Considerations

1. **HTTPS:** Всегда используйте HTTPS в production
2. **Secrets:** Храните секреты в environment variables или Vault
3. **Firewall:** Откройте только порт 3000 (или 443 для HTTPS)
4. **Updates:** Регулярно обновляйте образы/бинарники

### B. Performance Tuning

**Рекомендуемые лимиты:**
- CPU: 2 cores minimum
- Memory: 512MB minimum, 2GB recommended
- Disk: 10GB minimum для БД и логов

### C. Contact & Support

- GitHub: https://github.com/tnl-o/semarust
- Issues: https://github.com/tnl-o/semarust/issues
- Documentation: https://github.com/tnl-o/semarust/tree/main/docs

---

*Последнее обновление: 17 марта 2026 г.*
