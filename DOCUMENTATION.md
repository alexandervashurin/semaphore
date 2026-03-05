# 🗂 Документация Semaphore UI

## ✅ Актуальная документация

### Основная
| Файл | Описание |
|------|----------|
| [README.md](../README.md) | Быстрый старт и основы |
| [CONFIG.md](../CONFIG.md) | Переменные окружения и конфигурация |
| [API.md](../API.md) | REST API документация |
| [AUTH.md](../AUTH.md) | Аутентификация и авторизация |
| [CHANGELOG.md](../CHANGELOG.md) | История изменений |

### Развёртывание
| Файл | Описание |
|------|----------|
| [DOCKER_DEMO.md](../DOCKER_DEMO.md) | Docker демонстрация |
| [scripts/README.md](../scripts/README.md) | Скрипты запуска |
| [db/postgres/DEMO.md](../db/postgres/DEMO.md) | PostgreSQL с демо-данными |
| [deployment/docker/README.md](../deployment/docker/README.md) | Docker развёртывание |
| [deployment/systemd/README.md](../deployment/systemd/README.md) | Systemd развёртывание |

### Безопасность
| Файл | Описание |
|------|----------|
| [SECURITY.md](../SECURITY.md) | Политика безопасности |
| [SECURITY_AUDIT_2026_02_28.md](../SECURITY_AUDIT_2026_02_28.md) | Аудит безопасности |

### Разработка
| Файл | Описание |
|------|----------|
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Руководство для контрибьюторов |
| [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) | Кодекс поведения |
| [MIDDLEWARE.md](../MIDDLEWARE.md) | Middleware компоненты |

---

## ⚠️ Устаревшая документация (историческая)

Эти файлы содержат информацию о завершённой миграции и не актуальны для пользователей:

### Отчёты о миграции (rust/)
- `rust/MIGRATION_*.md` - все файлы о миграции
- `rust/API_MIGRATION_COMPLETE.md` - отчёт о миграции API
- `rust/CLI_MIGRATION_COMPLETE_FINAL.md` - отчёт о миграции CLI
- `rust/CONFIG_DECOMPOSITION_FINAL.md` - отчёт о миграции конфигурации
- `rust/BOLTDB_DECOMPOSITION.md` - отчёт о удалении BoltDB
- `rust/GO_MODULES_REMOVAL_GUIDE.md` - руководство по удалению Go модулей
- `rust/HANDLERS_DECOMPOSITION.md` - декомпозиция handlers
- `rust/LOCAL_JOB_RUST_COMPLETE.md` - реализация Local Job
- `rust/UTIL_CONFIG_MIGRATION_COMPLETE.md` - миграция util config

### Планы работ (выполнены)
- `PORTING_PLAN.md` - план портирования (завершён)
- `PLAN_FURTHER_WORK.md` - план дальнейших работ (устарел)
- `BUILD_FIX_PLAN.md` - план исправления ошибок (завершён)
- `BUILD_ERRORS.md` - ошибки сборки (исправлены)
- `CRUD_COMPLETE.md` - CRUD операции (завершены)

### Дублирующая документация
- `QUICK_START.md` - дублирует README.md
- `QUICK_START_DEMO.md` - дублирует db/postgres/DEMO.md
- `POSTGRES_SETUP.md` - дублирует scripts/README.md

---

## 📊 Статус проекта

**Миграция с Go на Rust:** ✅ **100% ЗАВЕРШЕНА**

| Компонент | Статус |
|-----------|--------|
| Backend (Rust) | ✅ Готов |
| Frontend (Vue 2) | ✅ Готов |
| SQLite | ✅ Готов |
| PostgreSQL | ✅ Готов |
| MySQL | ✅ Готов |
| API | ✅ 100% |
| CLI | ✅ 100% |
| Тесты | ✅ 503 passed |

---

## 🎯 Рекомендации

### Для пользователей
1. Используйте **README.md** для быстрого старта
2. Смотрите **CONFIG.md** для настройки переменных окружения
3. Используйте **DOCKER_DEMO.md** для демонстрационного окружения

### Для разработчиков
1. Изучите **CONTRIBUTING.md** перед внесением изменений
2. Смотрите **API.md** для понимания REST API
3. Используйте **MIDDLEWARE.md** для понимания архитектуры

### Для контрибьюторов
1. Прочтите **CODE_OF_CONDUCT.md**
2. Изучите **SECURITY.md** для политик безопасности
3. Проверьте **CHANGELOG.md** для последних изменений
