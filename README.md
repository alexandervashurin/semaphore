# Velum — Rust Edition

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org)
[![Build](https://github.com/tnl-o/velum/actions/workflows/rust.yml/badge.svg)](https://github.com/tnl-o/velum/actions)
[![Coverage](https://github.com/tnl-o/rust_semaphore/actions/workflows/coverage.yml/badge.svg)](https://github.com/tnl-o/rust_semaphore/actions/workflows/coverage.yml)
[![Release](https://img.shields.io/github/v/release/alexandervashurin/semaphore?label=latest)](https://github.com/alexandervashurin/semaphore/releases)

**Velum** — это система управления и автоматизации DevOps задач с открытым исходным кодом. Написана на Rust, управляет Ansible, Terraform, OpenTofu, Terragrunt, Bash и PowerShell через веб-интерфейс с базой данных PostgreSQL.

> **База данных:** Только PostgreSQL (SQLite/MySQL удалены в v2.2)
> **Тесты:** 710+ успешных тестов
> **Kubernetes:** Полная интеграция — 33 UI страницы, 60+ API endpoints, WebSocket streaming, Security & RBAC
> **Последний релиз:** [v2.5.1](https://github.com/alexandervashurin/semaphore/releases/tag/v2.5.1) — Kubernetes UI Production Ready

---

## ⚡ Quick Start (5 команд)

### Запуск Demo (одна команда)

```bash
docker compose -f docker-compose.demo.yml up
```

**Доступ:** http://localhost:8088  
**Логин/пароль:** `admin / admin123`

### Полный цикл (5 команд)

```bash
# 1. Запуск Demo-стенда
docker compose -f docker-compose.demo.yml up --build -d

# 2. Логин в систему (получение токена)
curl -X POST http://localhost:8088/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"auth":"admin","password":"admin123"}'

# 3. Создать проект
curl -X POST http://localhost:8088/api/projects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <TOKEN>" \
  -d '{"name":"My Project","type":"ansible"}'

# 4. Запустить задачу
curl -X POST http://localhost:8088/api/project/1/tasks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <TOKEN>" \
  -d '{"template_id":1}'

# 5. Получить лог задачи
curl -X GET http://localhost:8088/api/project/1/tasks/1/output \
  -H "Authorization: Bearer <TOKEN>"
```

---

## 📸 Скриншоты

### Dashboard
![Dashboard](docs/images/dashboard.png)
*Главная панель с проектами и задачами*

### Kubernetes Pods
![Kubernetes Pods](docs/images/k8s-pods.png)
*Управление Pod'ами с WebSocket логами*

### Template Editor
![Template Editor](docs/images/template-editor.png)
*Редактор шаблонов Ansible/Terraform*

### Task Log (WebSocket)
![Task Log](docs/images/task-log.png)
*Потоковая передача логов задачи в реальном времени*

### Workflow Builder
![Workflow Builder](docs/images/workflow-builder.png)
*Визуальный конструктор многошаговых пайплайнов*

### MCP Server (AI Integration)
![MCP Server](docs/images/mcp-server.png)
*Embedded MCP сервер для AI-интеграции*

---

---

## 🚀 Быстрый старт

### Вариант 1: Dev режим (рекомендуется для разработки)

**Требования:**
- Docker и Docker Compose
- Rust/Cargo (установлен)

```bash
# 1. Инициализация (запуск PostgreSQL + создание БД и админа)
./velum.sh init dev

# 2. Запуск сервера
./velum.sh start dev

# 3. Остановка
./velum.sh stop
```

**Доступ:** http://localhost:3000  
**Логин/пароль:** `admin / admin123`

---

### Вариант 2: Docker (всё в контейнерах)

**Требования:**
- Docker и Docker Compose

```bash
# Запуск всех сервисов в Docker
./velum.sh start docker

# Остановка
./velum.sh stop
```

**Доступ:** http://localhost  
**Логин/пароль:** `admin / demo123`

---

### Вариант 3: Ручной запуск (для разработки)

```bash
# 1. Запуск PostgreSQL (Docker)
docker compose -f docker-compose.postgres.yml up -d

# 2. Инициализация БД
export SEMAPHORE_DB_DIALECT=postgres
export SEMAPHORE_DB_URL=postgres://semaphore:semaphore123@localhost:5432/semaphore

cd rust
cargo run -- user add --username admin --name "Administrator" --email admin@localhost --password admin123 --admin

# 3. Запуск сервера
cargo run -- server --host 0.0.0.0 --port 3000
```

**Доступ:** http://localhost:3000  
**Логин/пароль:** `admin / admin123`

---

## 📦 Установка

### Из релиза GitHub

Скачайте бинарник для вашей платформы из релизов:

**Основной репозиторий:**
```bash
wget https://github.com/tnl-o/velum/releases/download/v2.4.0/velum-linux-x86_64
chmod +x velum-linux-x86_64
./velum-linux-x86_64 server --host 0.0.0.0 --port 3000
```

**Текущий форк (актуальная версия):**
```bash
wget https://github.com/alexandervashurin/semaphore/releases/download/v2.4.0/velum-linux-x86_64
chmod +x velum-linux-x86_64
./velum-linux-x86_64 server --host 0.0.0.0 --port 3000
```

### Из исходного кода

```bash
# Клонируйте репозиторий (основной или форк)
# Основной: git clone https://github.com/tnl-o/velum.git
# Форк:     git clone https://github.com/alexandervashurin/semaphore.git
git clone https://github.com/tnl-o/velum.git
cd velum

# Соберите проект
cd rust
cargo build --release

# Запустите
./target/release/velum server --host 0.0.0.0 --port 3000
```

---

## 📖 Все команды velum.sh

| Команда | Описание |
|---------|----------|
| `./velum.sh start [dev\|docker]` | Запуск сервиса в выбранном режиме |
| `./velum.sh stop` | Остановка всех сервисов |
| `./velum.sh restart` | Перезапуск сервисов |
| `./velum.sh clean` | Очистка данных (БД, volumes) |
| `./velum.sh init dev` | Инициализация БД (миграции + создание админа) |
| `./velum.sh status` | Показать статус сервисов |
| `./velum.sh logs` | Показать логи в реальном времени |
| `./velum.sh build` | Сборка проекта (backend + frontend) |
| `./velum.sh demo [dev]` | Запуск с демо-данными |
| `./velum.sh help` | Показать справку |

---

## ☸️ Kubernetes интеграция (v2.5+)

**33 страницы для управления кластером — Production Ready:**

| Категория | Страницы | Возможности |
|-----------|----------|-------------|
| **Workloads** | Pods, Deployments, ReplicaSets, DaemonSets, StatefulSets | CRUD, scale, restart, **pause/resume**, **rollback**, detailed history, evict API (PDB-aware) |
| **Networking** | Services, Ingress, NetworkPolicy, Gateway API | CRUD, EndpointSlices, selector matching, TLS |
| **Config** | ConfigMaps, Secrets | CRUD, YAML editor, validate, references, **masked values**, reveal endpoint |
| **Storage** | PV, PVC, StorageClass, Snapshots | CRUD, binding, CSI drivers, default badge |
| **Batch** | Jobs, CronJobs, PriorityClass, PDB | CRUD, suspend/resume, history, retry |
| **RBAC** | ServiceAccounts, Roles, Bindings, ClusterRoles | CRUD, **can-i cache**, PSA labels |
| **Advanced** | CRD, HPA, VPA, Quota, LimitRange | CRUD, dynamic objects, metrics hints |
| **Observability** | Metrics, Events, Topology | **WebSocket streaming**, CPU/Memory charts, Service→Deployment→Pod graph |
| **Helm** | Releases, Repos, Charts | Install, upgrade, rollback, uninstall, values editor |
| **Enterprise** | Audit, Backup, GitOps, Multi-cluster | Export CSV/JSON, Velero, ArgoCD/Flux, cluster switcher |
| **Tools** | Apply, Diff, Runbooks, Inventory Sync | YAML apply, **dry-run=server**, kubectl generator |

**Особенности:**
- ✅ **Exec terminal** — WebSocket с timeout 5 мин + heartbeat
- ✅ **Port-forward** — WebSocket туннель с timeout 10 мин
- ✅ **Logs streaming** — WebSocket /logs/stream endpoint
- ✅ **RBAC can-i кэш** — 5 мин TTL + frontend helpers
- ✅ **Audit logging** — деструктивные операции + export
- ✅ **Rate limiting** — WebSocket 60/min, burst 10/sec
- ✅ **Secrets masking** — *** вместо значений + reveal
- ✅ **Multi-cluster** — переключатель кластеров
- ✅ **i18n** — EN/RU + Dark/Light theme

**Backend API (~8000+ строк Rust):**
- 60+ REST endpoints (`/api/kubernetes/...`)
- WebSocket streaming (logs, events, exec, port-forward)
- kube-rs client (kube 0.98, k8s-openapi 0.24, v1_30)
- Routes декомпозиция (12 модулей)
- Security: audit, RBAC, rate limiting, timeouts

**Frontend (~5000+ строк Vanilla JS):**
- 33 Kubernetes страницы (k8s-*.html)
- Material Design UI
- WebSocket real-time
- Namespace picker + фильтры
- Status badges + detailed panels

---

## 🔧 Конфигурация

Все настройки задаются через переменные окружения:

| Переменная | Описание | Пример |
|------------|----------|--------|
| `SEMAPHORE_DB_DIALECT` | Тип БД (postgres) | `postgres` |
| `SEMAPHORE_DB_URL` | Строка подключения к БД | `postgres://user:pass@host:5432/db` |
| `SEMAPHORE_WEB_PATH` | Путь к фронтенду | `./web/public` |
| `SEMAPHORE_ADMIN` | Имя авто-созданного админа | `admin` |
| `SEMAPHORE_ADMIN_PASSWORD` | Пароль админа | `admin123` |
| `SEMAPHORE_ADMIN_EMAIL` | Email админа | `admin@localhost` |
| `SEMAPHORE_ACCESS_KEY_ENCRYPTION` | Ключ шифрования AES-256 | `my-secret-key-32-chars-long` |
| `RUST_LOG` | Уровень логирования | `info`, `debug`, `warn` |
| `MCP_TRANSPORT` | Транспорт для MCP | `stdio` или `http` |
| `MCP_PORT` | Порт для MCP HTTP | `8500` |

**Полная документация:** [`docs/technical/CONFIG.md`](docs/technical/CONFIG.md)

---

## 📋 Возможности

### Ядро автоматизации

- Запуск Ansible playbooks, Terraform/OpenTofu планов, Bash, PowerShell, Terragrunt
- Потоковая передача логов через WebSocket во время выполнения задач
- История задач с полным выводом для каждого запуска
- **Dry Run режим** — проверка без выполнения
- **Terraform Plan Preview** — показ плана перед выполнением
- **Diff view** — сравнение между двумя запусками
- **Task Snapshots & Rollback** — откат к предыдущему состоянию

### Проектные ресурсы

- **Templates** — определение задач с inventory, ключами и окружением
- **Inventories** — статические YAML/INI, динамические скрипты, Terraform workspace
- **Repositories** — Git checkout по ветке, тегу или коммиту
- **Access Keys** — SSH ключи, API токены, логины/пароли (AES-256-GCM)
- **Environments** — переменные с маскированием секретов
- **Schedules** — cron расписания и одноразовые запуски
- **Webhooks** — входящие HTTP вебхуки
- **Custom Credential Types** — типы учётных данных как в AWX

### Оркестрация workflow

- **Workflow Builder (DAG)** — многошаговые пайплайны с графом зависимостей
- **Template Marketplace** — 11 шаблонов сообщества (Nginx, Docker, K8s, мониторинг)
- **GitOps Drift Detection** — обнаружение дрейфа конфигурации
- **Terraform Cost Tracking** — интеграция Infracost для оценки стоимости

### Команда и доступ

- Мультипроектная архитектура с членами на проект
- Ролевая модель: Owner, Manager, Task Runner, Viewer
- Кастомные роли с битовыми масками разрешений
- Приглашения участников со ссылками
- **LDAP Groups → Teams auto-sync**
- Audit log всех действий пользователей

### Аутентификация

- Session login с хешированием bcrypt
- JWT токены для API доступа
- **TOTP 2FA** (RFC 6238, совместим с Google Authenticator/Authy)
- TOTP recovery коды
- LDAP аутентификация с синхронизацией групп
- OIDC / OAuth2 login

### Операции

- Backup и restore: полный экспорт/импорт проекта в JSON
- Secret Storages: HashiCorp Vault и DVLS интеграция
- Runners: саморегистрируемые агенты с heartbeat
- Apps: настраиваемые исполнители (Ansible, Terraform, Bash, Python, PowerShell, Pulumi, Terragrunt)
- Analytics dashboard (счётчики задач, success rate, timeline)
- **Notification Policies** — Slack, Microsoft Teams, PagerDuty, webhook
- **AI Integration** — анализ ошибок и генерация playbook
- **Embedded MCP server** — 60 инструментов для AI-native DevOps
- **Developer CLI** — `velum` бинарник для скриптов и CI

---

## 🛠️ Технологический стек

| Компонент | Технология |
|-----------|------------|
| **Runtime** | Rust stable, Tokio 1 |
| **Web framework** | Axum 0.8 (с WebSocket) |
| **Database** | SQLx 0.8, PostgreSQL |
| **Kubernetes** | kube 0.98, k8s-openapi 0.24 |
| **Frontend** | Vanilla JS, Material Design, Roboto |
| **Auth** | JWT (jsonwebtoken 9), bcrypt, HMAC-SHA1 TOTP, ldap3, OIDC |
| **Encryption** | AES-256-GCM (aes-gcm 0.10) |
| **Scheduler** | cron 0.15 |
| **CI** | GitHub Actions — build, clippy, test |

---

## 📁 Структура репозитория

```
├── rust/                   Backend — Rust / Axum / SQLx / Kubernetes
│   └── src/
│       ├── api/            HTTP handlers (200+ функций)
│       │   └── handlers/
│       │       └── kubernetes/  K8s API: pods, deployments, workloads
│       ├── models/         Модели данных
│       ├── db/             Слой БД (PostgreSQL)
│       ├── services/       Бизнес-логика (task runner, scheduler, backup)
│       ├── kubernetes/     K8s клиент, Helm, Jobs
│       └── config/         Загрузка конфигурации
├── web/public/             Frontend — 48 HTML страниц, Vanilla JS
│   ├── k8s-pods.html       Kubernetes Pods UI с WebSocket логами
│   ├── k8s-deployments.html Deployments UI с scale/restart/rollback
│   ├── k8s-configmaps.html ConfigMaps CRUD с JSON редактором
│   ├── k8s-secrets.html    Secrets CRUD с base64 decode
│   ├── k8s-jobs.html       Jobs, CronJobs, PDB management
│   └── ...                 20+ Kubernetes страниц всего
├── mcp/                    Embedded MCP сервер (Rust)
├── db/postgres/            Скрипты миграций PostgreSQL
├── deploy/
│   ├── demo/               Demo стек (одна команда, порт 8088)
│   ├── dev/                Development стек (hot-reload)
│   └── prod/               Production стек (Nginx, сети)
├── docs/
│   ├── technical/          API, Auth, Config, Security, Webhooks
│   ├── guides/             Setup, Testing, Demo, Troubleshooting
│   ├── releases/           Release notes
│   ├── future/             Roadmap и планы
│   └── archive/            Исторические отчёты
├── scripts/                Утилиты и SQL seeds
├── demo-playbooks/         Примеры Ansible playbooks
├── Dockerfile              Production multi-stage image
└── docker-compose.yml      Полный стек (PostgreSQL + backend)
```

---

## 📚 Документация

| Документ | Описание |
|----------|----------|
| [ЗАПУСК.md](ЗАПУСК.md) | Руководство по запуску (подробное) |
| [docs/technical/API.md](docs/technical/API.md) | REST API справочник |
| [docs/technical/AUTH.md](docs/technical/AUTH.md) | Аутентификация: JWT, TOTP, LDAP, OIDC |
| [docs/technical/CONFIG.md](docs/technical/CONFIG.md) | Переменные окружения |
| [docs/technical/BACKUP_RESTORE.md](docs/technical/BACKUP_RESTORE.md) | Backup и restore |
| [docs/guides/TROUBLESHOOTING.md](docs/guides/TROUBLESHOOTING.md) | Частые проблемы |
| [docs/future/ROADMAP.md](docs/future/ROADMAP.md) | Roadmap |
| [CHANGELOG.md](CHANGELOG.md) | История версий |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Руководство по контрибьюции |

---

## 🧪 Разработка

```bash
cd rust

# Проверка компиляции
cargo check

# Линтер (0 предупреждений)
cargo clippy -- -D warnings

# Тесты (710+)
cargo test

# Запуск сервера
cargo run -- server --host 0.0.0.0 --port 3000

# Версия
cargo run -- version

# Сборка release
cargo build --release
```

---

## 📦 Релизы

| Версия | Дата | Ключевые изменения |
|--------|------|-------------------|
| **v2.5.1** | 2026-04-01 | **Bugfix** — устранены предупреждения компиляции, clippy warnings |
| **v2.5.0** | 2026-04-01 | **Kubernetes UI Release** — 33 страницы, Security & RBAC, Helm, Multi-cluster, Audit |
| **v2.4.0** | 2026-03 | Kubernetes Workloads Complete — 20 страниц, WebSocket, ~4800 строк кода |
| **v2.3.0** | 2026-03 | Kubernetes Workloads API — ReplicaSets, DaemonSets, StatefulSets |
| **v2.2.0** | 2026-02 | MCP server + Pods/Deployments API |
| **v2.1.0** | 2026-02 | PostgreSQL-only, 710+ тестов |
| **v2.0.0** | 2026-01 | Initial Rust release |

**Последний релиз:** [v2.5.1](https://github.com/alexandervashurin/semaphore/releases/tag/v2.5.1) — Kubernetes UI Production Ready

---

## 🔗 Ссылки

| Ресурс | Ссылка |
|--------|--------|
| **Основной репозиторий** | https://github.com/tnl-o/velum |
| **Текущий форк** | https://github.com/alexandervashurin/semaphore |
| **Go оригинал** | https://github.com/semaphoreui/semaphore |
| **Документация** | https://github.com/tnl-o/velum/wiki |

---

## 📄 Лицензия

MIT © [Alexander Vashurin](https://github.com/alexandervashurin)
