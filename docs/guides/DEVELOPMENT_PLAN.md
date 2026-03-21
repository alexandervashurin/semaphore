# 📋 Единый план разработки Velum

> **Последнее обновление:** 14 марта 2026 г.  
> **Версия:** v2.0.0  
> **Статус:** Q4 2026 завершён

---

## 📊 Текущее состояние проекта

### ✅ Завершено (Q1-Q4 2026)

| Категория | Статус | Описание |
|-----------|--------|----------|
| **Backend (Rust)** | ✅ 100% | 306 файлов, ~57,000 строк кода |
| **Frontend (Vanilla JS)** | ✅ 100% | 27 файлов, ~8,500 строк |
| **API Endpoints** | ✅ 100+ | REST + GraphQL + WebSocket |
| **База данных** | ✅ 100% | SQLite, PostgreSQL, MySQL |
| **Docker** | ✅ 100% | 4 варианта образов (до 92% оптимизация) |
| **Тесты** | ✅ 655 passed | Integration + Unit + E2E |
| **Плагины** | ✅ 100% | 7 типов, 40+ хуков |
| **Аналитика** | ✅ 100% | Prometheus + Dashboard |
| **Audit Log** | ✅ 100% | 50+ типов событий |
| **Webhooks** | ✅ 100% | 5 типов, история отправок |

### 📦 Реализованные модули

1. **Аутентификация**: JWT, OAuth2/OIDC, LDAP, TOTP 2FA
2. **Проекты**: Полный CRUD, команда, настройки
3. **Шаблоны**: Ansible, Terraform, Shell, Build, Deploy
4. **Задачи**: Запуск, мониторинг, логи, real-time статус
5. **Playbooks**: CRUD, синхронизация с Git, запуск
6. **Инвентари**: Hosts, группы, переменные
7. **Репозитории**: Git интеграция, SSH ключи
8. **Окружения**: Переменные, секреты
9. **Ключи доступа**: SSH, Vault, Login, Certificate
10. **Расписания**: Cron, включение/отключение
11. **Интеграции**: Generic, GitHub, GitLab, Jenkins
12. **Webhooks**: Generic, Slack, Teams, Discord, Telegram
13. **Audit Log**: Поиск, фильтрация, экспортирование
14. **Аналитика**: Дашборды, графики, метрики
15. **Плагины**: WASM загрузчик, менеджер плагинов
16. **GraphQL API**: Query, Mutation, Subscription
17. **gRPC API**: 3 сервиса для внутренней коммуникации
18. **Telegram Bot**: Команды /start, /help, уведомления

---

## 🎯 План разработки v2.1.0 (Март-Апрель 2026)

### 🔴 Критические задачи (Must Have)

#### 1. Тестирование и качество

| Задача | Оценка | Статус |
|--------|--------|--------|
| Unit тесты backend (цель: >80%) | 5 дней | 🔄 67% |
| Integration тесты API | 3 дня | ⏳ Ожидает |
| E2E тесты frontend (Playwright) | 4 дня | ⏳ Ожидает |
| Security аудит (cargo audit) | 2 дня | ⏳ Ожидает |
| Performance тесты | 2 дня | ⏳ Ожидает |

**Файлы:**
- `rust/src/*/tests.rs` — Unit тесты
- `test-schedules-api.sh` — Integration тесты ✅
- `test-playbook-runs-api.sh` — Integration тесты ✅
- `test/e2e/` — E2E тесты (Playwright)

---

#### 2. Документация

| Задача | Оценка | Статус |
|--------|--------|--------|
| OpenAPI спецификация | 2 дня | ⏳ Ожидает |
| User Guide | 3 дня | ⏳ Ожидает |
| Admin Guide | 2 дня | ⏳ Ожидает |
| Developer Guide | 3 дня | ⏳ Ожидает |
| Deployment Guide | 2 дня | ⏳ Ожидает |

**Файлы для обновления:**
- `api-docs.yml` — OpenAPI спецификация
- `docs/user/` — Руководство пользователя
- `docs/admin/` — Руководство администратора
- `docs/deployment/` — Деплой

---

#### 3. Безопасность

| Задача | Оценка | Статус |
|--------|--------|--------|
| Rate limiting (завершить) | 2 дня | ⏳ Ожидает |
| CORS настройка (production) | 1 день | ⏳ Ожидает |
| Security headers (CSP, HSTS) | 1 день | ⏳ Ожидает |
| Secrets management (Vault) | 2 дня | ⏳ Ожидает |

**Файлы:**
- `rust/src/api/middleware/rate_limiter.rs`
- `rust/src/api/middleware/security_headers.rs`

---

### 🟠 Важные задачи (Should Have)

#### 4. Frontend улучшения

| Задача | Оценка | Статус |
|--------|--------|--------|
| Analytics UI (Vue.js версия) | 3 дня | ⏳ Ожидает |
| Webhooks UI (Vue.js версия) | 2 дня | ⏳ Ожидает |
| Audit Log UI (Vue.js версия) | 2 дня | ⏳ Ожидает |
| Plugin Management UI | 2 дня | ⏳ Ожидает |

**Файлы:**
- `web/src/views/project/Analytics.vue`
- `web/src/views/project/Webhooks.vue`
- `web/src/views/project/AuditLog.vue`
- `web/src/views/admin/Plugins.vue`

**Примечание:** Vanilla JS версии уже реализованы ✅

---

#### 5. Q4 2026 — Завершение

| Задача | Оценка | Статус |
|--------|--------|--------|
| Telegram Bot (уведомления) | 3 дня | ⏳ Ожидает |
| GraphQL API (полные мутации) | 2 дня | ✅ Завершено |
| GraphQL Subscription | 2 дня | ✅ Завершено |
| Prometheus alerts | 2 дня | ⏳ Ожидает |

---

#### 6. Production готовность

| Задача | Оценка | Статус |
|--------|--------|--------|
| Health checks (complete) | 1 день | ⏳ Ожидает |
| Grafana dashboards | 2 дня | ⏳ Ожидает |
| Log aggregation (Loki) | 2 дня | ⏳ Ожидает |
| Backup strategy (авто) | 2 дня | ⏳ Ожидает |
| Disaster recovery plan | 2 дня | ⏳ Ожидает |

---

### 🟢 Желательные задачи (Nice to Have)

#### 7. Масштабирование (Q1 2027)

| Задача | Оценка | Статус |
|--------|--------|--------|
| Redis кэширование | 3 дня | 🔮 Будущее |
| Cluster mode | 7 дней | 🔮 Будущее |
| Horizontal scaling | 5 дней | 🔮 Будущее |
| Load balancing | 3 дней | 🔮 Будущее |

---

#### 8. Интеграции

| Задача | Оценка | Статус |
|--------|--------|--------|
| Kubernetes (Helm chart) | 7 дней | 🔮 Будущее |
| Terraform provider | 7 дней | 🔮 Будущее |
| Prometheus exporter | 2 дня | 🔮 Будущее |
| Grafana dashboards | 3 дня | 🔮 Будущее |

---

#### 9. Дополнительные платформы

| Задача | Оценка | Статус |
|--------|--------|--------|
| Desktop app (Tauri) | 10 дней | 🔮 Будущее |
| Mobile app (React Native) | 14 дней | 🔮 Будущее |
| CLI improvements | 3 дня | 🔮 Будущее |

---

## 📅 Рекомендуемый план релизов

### v2.0.0 — Major Release (Март 2026) ✅

**Выпущено:** 14 марта 2026 г.

**Что включено:**
- ✅ Analytics API & UI
- ✅ Webhooks Management UI
- ✅ Schedules CRUD
- ✅ Playbook Runs UI
- ✅ Integration Tests
- ✅ GraphQL CRUD мутации
- ✅ GraphQL Subscription
- ✅ Vanilla JS Frontend (100%)

---

### v2.1.0 — Stability Release (Апрель 2026)

**Фокус:** Тестирование, безопасность, документация

**План:**
1. Unit тесты (>80% coverage)
2. Integration тесты (все API endpoints)
3. E2E тесты (критические сценарии)
4. Security аудит
5. OpenAPI спецификация
6. User Guide
7. Rate limiting (завершение)
8. Security headers

**Срок:** 2-3 недели

---

### v2.2.0 — Production Release (Май 2026)

**Фокус:** Production готовность

**План:**
1. Health checks (complete)
2. Backup strategy
3. Monitoring (Grafana, Loki)
4. Disaster recovery plan
5. Performance оптимизация
6. Production документация

**Срок:** 3-4 недели

---

### v2.3.0 — Enhanced Release (Июнь 2026)

**Фокус:** Расширенные функции

**План:**
1. Telegram Bot (уведомления)
2. Prometheus alerts
3. Plugin Management UI
4. Advanced Analytics
5. Mobile responsive UI

**Срок:** 4-5 недель

---

### v3.0.0 — Enterprise Release (Q3-Q4 2026)

**Фокус:** Масштабирование и интеграции

**План:**
1. Redis кэширование
2. Cluster mode
3. Kubernetes integration
4. Terraform provider
5. Desktop app (Tauri)

**Срок:** 3-4 месяца

---

## 🗂️ Структура документации

### ✅ Актуальные файлы

| Файл | Описание | Статус |
|------|----------|--------|
| `README.md` | Главная документация | ✅ Актуален |
| `ROADMAP.md` | Дорожная карта | ✅ Актуален |
| `CHANGELOG.md` | История изменений | ✅ Актуален |
| `API.md` | API документация | ✅ Актуален |
| `GRAPHQL_API.md` | GraphQL API | ✅ Актуален |
| `ANALYTICS.md` | Аналитика | ✅ Актуален |
| `AUDIT_LOG.md` | Audit Log | ✅ Актуален |
| `WEBHOOK.md` | Webhooks | ✅ Актуален |
| `PLUGINS.md` | Плагины | ✅ Актуален |
| `AUTH.md` | Аутентификация | ✅ Актуален |
| `CONFIG.md` | Конфигурация | ✅ Актуален |
| `SEMAPHORE_SH.md` | Скрипт запуска | ✅ Актуален |
| `ЗАПУСК_ДЕМО.md` | Демо режим | ✅ Актуален |
| `CRUD_DEMO.md` | CRUD демо | ✅ Актуален |
| `TROUBLESHOOTING.md` | Решение проблем | ✅ Актуален |

---

### ⚠️ Файлы требующие обновления

| Файл | Проблема | Действие |
|------|----------|----------|
| `DOCKER_DEMO.md` | Устаревшие команды | Обновить |
| `DEMO_DATA.md` | Устаревшие команды | Обновить |
| `db/postgres/DEMO_MODE.md` | 1 устаревшая ссылка | Обновить |
| `CRUD_TESTS.md` | 2 устаревшие ссылки | Обновить |

---

### 🗑️ Файлы на удаление (дубли)

| Файл | Дублирует | Действие |
|------|-----------|----------|
| `ROADMAP_DETAILED.md` | `ROADMAP.md` | ❌ Удалить |
| `PROJECT_COMPLETION_PLAN.md` | `ROADMAP.md` | ❌ Удалить |
| `FRONTEND_PLAN.md` | `ROADMAP.md` | ❌ Удалить |
| `PLAYBOOK_ROADMAP.md` | `PLAYBOOK_*.md` | ❌ Удалить |
| `DOCUMENTATION_UPDATE_PLAN.md` | Завершено | ❌ Удалить |
| `SCRIPTS.md` | `SEMAPHORE_SH.md` | ❌ Удалить |

---

## 📊 Сводная таблица работ

### Для v2.1.0 (2-3 недели)

| Категория | Задач | Дней | Приоритет |
|-----------|-------|------|-----------|
| Тестирование | 5 | 16 | 🔴 Критический |
| Документация | 5 | 12 | 🔴 Критический |
| Безопасность | 4 | 6 | 🔴 Критический |
| **Итого** | **14** | **34** | **6-7 недель** |

### Для v2.2.0 (3-4 недели)

| Категория | Задач | Дней | Приоритет |
|-----------|-------|------|-----------|
| Production готовность | 5 | 10 | 🟠 Высокий |
| Monitoring | 3 | 6 | 🟠 Высокий |
| **Итого** | **8** | **16** | **3-4 недели** |

### Для v3.0.0 (3-4 месяца)

| Категория | Задач | Дней | Приоритет |
|-----------|-------|------|-----------|
| Масштабирование | 4 | 18 | 🟢 Средний |
| Интеграции | 4 | 19 | 🟢 Средний |
| Платформы | 3 | 17 | 🟢 Средний |
| **Итого** | **11** | **54** | **3-4 месяца** |

---

## 🎯 Минимальный план для v2.1.0

Если нужно быстро выпустить стабильную версию:

### Критический минимум (2 недели)

1. **Тестирование** (5 дней)
   - Unit тесты для критических модулей
   - Integration тесты для API
   - Security аудит

2. **Документация** (3 дня)
   - OpenAPI спецификация
   - Quick start guide
   - Deployment guide

3. **Безопасность** (2 дня)
   - Rate limiting (завершение)
   - Security headers
   - CORS настройка

**Итого: 10 дней = 2 недели**

---

## 📝 Приоритеты разработки

### Высокий приоритет (Март-Апрель 2026)

1. ✅ GraphQL API (CRUD + Subscription)
2. ✅ Integration тесты (Schedules, Playbook Runs)
3. ✅ Analytics UI (Vanilla JS)
4. ✅ Webhooks UI (Vanilla JS)
5. ⏳ Unit тесты (>80% coverage)
6. ⏳ OpenAPI спецификация
7. ⏳ Rate limiting (завершение)
8. ⏳ Security headers

### Средний приоритет (Май-Июнь 2026)

1. Production готовность
2. Monitoring (Grafana, Loki)
3. Backup strategy
4. Telegram Bot (уведомления)
5. Plugin Management UI

### Низкий приоритет (Q3-Q4 2026)

1. Redis кэширование
2. Cluster mode
3. Kubernetes integration
4. Terraform provider
5. Desktop/Mobile app

---

## 💡 Рекомендации

### Для быстрого релиза (2-3 недели)

1. **Сфокусироваться на тестировании**
   - Unit тесты для критических путей
   - Integration тесты для API
   - E2E тесты для основных сценариев

2. **Завершить безопасность**
   - Rate limiting
   - Security headers
   - CORS настройка

3. **Минимальная документация**
   - OpenAPI спецификация
   - Quick start guide
   - Deployment instructions

### Для полноценного релиза (6-8 недель)

1. **Полное тестирование**
   - >80% code coverage
   - Performance тесты
   - Security audit

2. **Полная документация**
   - User Guide
   - Admin Guide
   - Developer Guide

3. **Production инструменты**
   - Monitoring
   - Alerting
   - Backup/Recovery

---

## 🚀 Следующие шаги

### Немедленно (эта неделя)

1. ⏳ Начать unit тестирование backend
2. ⏳ Создать OpenAPI спецификацию
3. ⏳ Реализовать rate limiting (завершение)
4. ⏳ Реализовать security headers

### В этом месяце

1. Завершить критические задачи v2.1.0
2. Провести security аудит
3. Написать документацию
4. Подготовить v2.1.0 RC

### В следующем квартале

1. Выпустить v2.1.0 Stability
2. Начать работу над v2.2.0 Production
3. Планирование v3.0.0 Enterprise

---

## 📚 Удаление дублирующей документации

### Файлы для удаления

```bash
# Удалить дубли планов
rm ROADMAP_DETAILED.md
rm PROJECT_COMPLETION_PLAN.md
rm FRONTEND_PLAN.md
rm PLAYBOOK_ROADMAP.md
rm DOCUMENTATION_UPDATE_PLAN.md
rm SCRIPTS.md
```

### Файлы для обновления

```bash
# Обновить устаревшие ссылки
# DOCKER_DEMO.md - заменить команды запуска
# DEMO_DATA.md - заменить команды запуска
# db/postgres/DEMO_MODE.md - 1 ссылка
# CRUD_TESTS.md - 2 ссылки
```

---

*Этот документ является единственным источником истины для плана разработки*  
*Все остальные планы устарели и подлежат удалению*

*Последнее обновление: 14 марта 2026 г.*
