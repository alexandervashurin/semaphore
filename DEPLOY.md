# DEPLOY — Руководство по развёртыванию Velum

Velum — Rust/Axum переписывание [Semaphore](https://github.com/semaphoreui/semaphore).
Поддерживает SQLite (demo/dev) и PostgreSQL (prod).

---

## Быстрый старт — Demo (1 команда)

```bash
docker compose -f deploy/demo/docker-compose.yml up --build -d
```

Открыть: **http://localhost:8088** · Логин: **admin / admin123**

---

## Production развёртывание

### 1. PostgreSQL + Nginx + Docker Compose

**Требования:**
- Docker 20+
- Docker Compose 2+
- PostgreSQL 13+
- Nginx 1.20+

#### Шаг 1: Подготовка окружения

```bash
# Создать директорию для развёртывания
mkdir -p /opt/velum
cd /opt/velum

# Скопировать файлы развёртывания
cp /path/to/velum/deploy/prod/* .
cp /path/to/velum/nginx.conf .
```

#### Шаг 2: Настройка переменных окружения

```bash
# Создать .env файл
cat > .env << EOF
# PostgreSQL
POSTGRES_USER=velum
POSTGRES_PASSWORD=<GENERATE_SECURE_PASSWORD>
POSTGRES_DB=velum

# Velum
VELUM_DB_DIALECT=postgres
VELUM_DB_URL=postgres://velum:<PASSWORD>@velum-db:5432/velum
VELUM_WEB_PATH=/app/web/public
VELUM_JWT_SECRET=<GENERATE_SECRET_32_CHARS>
VELUM_ADMIN=admin
VELUM_ADMIN_PASSWORD=<CHANGE_ADMIN_PASSWORD>
VELUM_ADMIN_EMAIL=admin@example.com

# Nginx
NGINX_HOST=velum.example.com
EOF

# Сгенерировать безопасный пароль для БД
openssl rand -base64 32

# Сгенерировать JWT secret (32 символа)
openssl rand -hex 32
```

#### Шаг 3: Настройка Nginx

```nginx
# /etc/nginx/sites-available/velum
server {
    listen 80;
    server_name velum.example.com;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

#### Шаг 4: Запуск

```bash
# Запустить все сервисы
docker compose up -d

# Проверить статус
docker compose ps

# Просмотреть логи
docker compose logs -f velum
```

---

### 2. Systemd сервис (нативная установка)

#### Шаг 1: Установка бинарного файла

```bash
# Скачать релиз
wget https://github.com/tnl-o/velum/releases/download/v2.5.1/velum-linux-x86_64
chmod +x velum-linux-x86_64
sudo mv velum-linux-x86_64 /usr/local/bin/velum

# Или собрать из исходников
cd /opt/velum/rust
cargo build --release
sudo cp target/release/velum /usr/local/bin/
```

#### Шаг 2: Создание пользователя и директорий

```bash
# Создать пользователя
sudo useradd --system --no-create-home --shell /bin/false velum

# Создать директории
sudo mkdir -p /var/lib/velum
sudo mkdir -p /etc/velum
sudo chown velum:velum /var/lib/velum
```

#### Шаг 3: Конфигурация

```bash
# Создать конфигурационный файл
sudo cat > /etc/velum/velum.conf << EOF
VELUM_DB_DIALECT=postgres
VELUM_DB_URL=postgres://velum:password@localhost:5432/velum
VELUM_WEB_PATH=/usr/share/velum/web/public
VELUM_JWT_SECRET=<YOUR_JWT_SECRET>
VELUM_ADMIN=admin
VELUM_ADMIN_PASSWORD=<ADMIN_PASSWORD>
RUST_LOG=info
EOF
```

#### Шаг 4: Systemd unit файл

```ini
# /etc/systemd/system/velum.service
[Unit]
Description=Velum Ansible Semaphore
Documentation=https://github.com/tnl-o/velum
After=network.target postgresql.service

[Service]
Type=notify
User=velum
Group=velum
EnvironmentFile=/etc/velum/velum.conf
ExecStart=/usr/local/bin/velum server --host 127.0.0.1 --port 3000
Restart=on-failure
RestartSec=5
LimitNOFILE=65535

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/velum

[Install]
WantedBy=multi-user.target
```

#### Шаг 5: Запуск сервиса

```bash
# Перезагрузить systemd
sudo systemctl daemon-reload

# Включить автозапуск
sudo systemctl enable velum

# Запустить сервис
sudo systemctl start velum

# Проверить статус
sudo systemctl status velum

# Просмотреть логи
journalctl -u velum -f
```

---

### 3. SSL/TLS с Let's Encrypt

#### Вариант A: Nginx + Certbot

```bash
# Установить Certbot
sudo apt install certbot python3-certbot-nginx

# Получить сертификат
sudo certbot --nginx -d velum.example.com

# Автоматическое обновление (добавить в cron)
0 3 * * * certbot renew --quiet
```

#### Вариант B: Docker + Traefik

```yaml
# docker-compose.prod.yml
services:
  traefik:
    image: traefik:v2.10
    command:
      - "--api.insecure=true"
      - "--providers.docker=true"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--certificatesresolvers.letsencrypt.acme.tlschallenge=true"
      - "--certificatesresolvers.letsencrypt.acme.email=admin@example.com"
      - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - "/var/run/docker.sock:/var/run/docker.sock:ro"
      - "letsencrypt:/letsencrypt"

  velum:
    image: velum:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.velum.rule=Host(`velum.example.com`)"
      - "traefik.http.routers.velum.entrypoints=websecure"
      - "traefik.http.routers.velum.tls.certresolver=letsencrypt"
    networks:
      - velum-net

volumes:
  letsencrypt:
```

#### Вариант C: Nginx Proxy Manager (UI)

```bash
docker run -d \
  -p 80:80 \
  -p 81:81 \
  -p 443:443 \
  -v nginx-proxy-manager-data:/data \
  -v nginx-proxy-manager-letsencrypt:/etc/letsencrypt \
  --name nginx-proxy-manager \
  jc21/nginx-proxy-manager:latest
```

---

## Мониторинг и обслуживание

### Health Check

```bash
# Проверка доступности API
curl -f http://localhost:3000/api/health || echo "API is down!"

# Проверка БД
docker compose exec velum-db pg_isready -U velum

# Проверка сервиса
systemctl is-active velum
```

### Backup

```bash
# Backup PostgreSQL
docker compose exec velum-db pg_dump -U velum velum > backup-$(date +%Y%m%d).sql

# Backup SQLite
cp /var/lib/velum/velum.db /backup/velum-$(date +%Y%m%d).db

# Полный backup проекта через API
curl -X GET http://localhost:3000/api/project/1/backup \
  -H "Authorization: Bearer <TOKEN>" \
  -o project-backup.json
```

### Restore

```bash
# Restore PostgreSQL
cat backup.sql | docker compose exec -T velum-db psql -U velum

# Restore SQLite
cp backup.db /var/lib/velum/velum.db
chown velum:velum /var/lib/velum/velum.db

# Restore проекта через API
curl -X POST http://localhost:3000/api/projects/restore \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <TOKEN>" \
  -d @project-backup.json
```

### Логирование

```bash
# Docker Compose
docker compose logs -f velum

# Systemd
journalctl -u velum -f

# Файл логов
tail -f /var/log/velum/velum.log
```

### Ротация логов

```ini
# /etc/logrotate.d/velum
/var/log/velum/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 velum velum
    sharedscripts
    postrotate
        systemctl reload velum > /dev/null 2>&1 || true
    endscript
}
```

---

## Безопасность

### Чеклист перед запуском

- [ ] Сменён пароль PostgreSQL (минимум 16 символов)
- [ ] Сгенерирован JWT_SECRET (минимум 32 символа)
- [ ] Сменён пароль администратора
- [ ] Настроен HTTPS (Let's Encrypt)
- [ ] PostgreSQL не экспонирован на публичный IP
- [ ] Настроен firewall (UFW/iptables)
- [ ] Включён SELinux/AppArmor
- [ ] Настроена ротация логов
- [ ] Настроен автоматический backup

### Firewall (UFW)

```bash
# Разрешить только необходимые порты
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 443/tcp   # HTTPS
sudo ufw enable

# Проверить статус
sudo ufw status verbose
```

### SELinux

```bash
# Проверить статус
getenforce

# Включить (если выключен)
sudo setenforce 1
sudo sed -i 's/SELINUX=permissive/SELINUX=enforcing/' /etc/selinux/config
```

---

## Варианты развёртывания

### 1. Demo — попробовать Velum

SQLite, без настройки, порт 8088.

```bash
cd deploy/demo
cp .env.example .env
docker compose up --build -d
```

Наполнить тестовыми данными:
```bash
bash fill-sqlite-demo-data.sh
```

---

### 2. Dev — разработка с hot-reload

PostgreSQL + cargo-watch. Backend перекомпилируется при изменении кода.

```bash
cd deploy/dev
cp .env.example .env
docker compose up -d
```

URL: http://localhost:3000

**Нативный запуск** (быстрее для разработки — только DB в Docker):

```bash
# Запустить только PostgreSQL
cd deploy/dev && docker compose up -d db

# Backend на хосте
cd rust
VELUM_DB_URL="postgres://velum:velum_dev_pass@localhost:5432/velum" \
VELUM_WEB_PATH=../web/public \
VELUM_ADMIN=admin VELUM_ADMIN_PASSWORD=admin123 \
cargo run -- server
```

---

### 3. DEB-пакет (Debian/Ubuntu)

```bash
# Сборка пакета
bash scripts/build-deb.sh

# Установка
sudo dpkg -i dist/velum_*.deb
sudo systemctl enable --now velum

# Конфиг: /etc/velum/velum.conf
# Логи:   journalctl -u velum -f
```

Подробнее: [docs/DEB_PACKAGE.md](docs/DEB_PACKAGE.md)

---

## Переменные окружения

| Переменная | По умолчанию | Описание |
|---|---|---|
| `VELUM_DB_DIALECT` | `sqlite` | Диалект БД: `sqlite` / `postgres` / `mysql` |
| `VELUM_DB_PATH` | `/tmp/velum.db` | Путь к SQLite-файлу |
| `VELUM_DB_URL` | — | Строка подключения PostgreSQL/MySQL |
| `VELUM_WEB_PATH` | `./web/public` | Путь к статическим файлам UI |
| `VELUM_TMP_PATH` | `/tmp/velum` | Временная папка для задач |
| `VELUM_JWT_SECRET` | `secret` | JWT-секрет (обязательно сменить!) |
| `VELUM_ADMIN` | — | Логин первого администратора |
| `VELUM_ADMIN_PASSWORD` | — | Пароль первого администратора |
| `VELUM_ADMIN_EMAIL` | — | Email первого администратора |
| `VELUM_LDAP_*` | — | LDAP-настройки (опционально) |
| `VELUM_OIDC_*` | — | OIDC-настройки (опционально) |
| `SEMAPHORE_AUTH_EMAIL_LOGIN_ENABLED` | `false` | `true` — в login metadata включается `emailEnabled` для UI |
| `RUST_LOG` | `info` | Уровень логирования |

В JSON-конфиге у объектов `auth.oidc_providers[]` можно задать поля **`email_claim`**, **`username_claim`**, **`name_claim`** (имена claims в OIDC userinfo; по умолчанию `email`, `preferred_username`, `name`). Также **`auth.emailLoginEnabled`** — то же, что переменная `SEMAPHORE_AUTH_EMAIL_LOGIN_ENABLED`.

---

## Обновление

```bash
# Получить последние изменения
git pull origin main

# Пересобрать и перезапустить (prod)
cd deploy/prod && docker compose up -d --build

# Demo
cd deploy/demo && docker compose up --build -d
```

---

## Структура проекта

```
velum/
├── rust/               # Backend — Rust (Axum, SQLx, Tokio)
│   └── src/
│       ├── api/        # HTTP handlers, middleware, routes
│       ├── db/         # Database adapters (SQLite, PostgreSQL, MySQL)
│       ├── services/   # Бизнес-логика (task runner, scheduler, auth)
│       └── cli/        # CLI команды (server, fill-demo-data, ...)
├── web/
│   └── public/         # Frontend — Vanilla JS + Material Design
│       ├── app.js      # API client, sidebar, утилиты
│       ├── styles.css  # Material Design CSS
│       └── *.html      # 30+ страниц UI
├── db/
│   ├── postgres/       # PostgreSQL: init.sql, migrations
│   └── migrations/     # SQLite migrations
├── deploy/             # Конфиги развёртывания
│   ├── demo/           # SQLite demo (docker-compose + .env.example)
│   ├── dev/            # Разработка с hot-reload
│   └── prod/           # Продакшен (PostgreSQL + Nginx)
├── docs/               # Документация
│   ├── technical/      # API, Auth, Config, Performance
│   ├── guides/         # Setup, Testing, Demo Data
│   ├── releases/       # Changelog, Release notes
│   ├── future/         # Roadmap, планируемые фичи
│   └── archive/        # Старые отчёты
├── scripts/            # Вспомогательные скрипты (build-deb, apply-migration)
└── DEPLOY.md           # ← этот файл
```

---

## Поддерживаемые БД

| БД | Статус | Использование |
|---|---|---|
| SQLite | ✅ Prod-ready | Demo, маленькие деплои |
| PostgreSQL 13+ | ✅ Prod-ready | Рекомендован для продакшена |
| MySQL 8+ | ✅ Поддерживается | Альтернатива PostgreSQL |

---

## Помощь

- [GitHub Issues](https://github.com/tnl-o/velum/issues)
- [docs/guides/TROUBLESHOOTING.md](docs/guides/TROUBLESHOOTING.md)
- [docs/technical/API.md](docs/technical/API.md)
- [docs/technical/CONFIG.md](docs/technical/CONFIG.md)
