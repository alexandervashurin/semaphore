# Changelog

Все заметные изменения в этом проекте будут задокументированы в этом файле.

Формат основан на [Keep a Changelog](https://keepachangelog.com/ru/1.0.0/),
и проект придерживается [Semantic Versioning](https://semver.org/lang/ru/).

## [Не выпущено]

### Добавлено

- Базовая структура проекта на Rust
- Модели данных (User, Project, Task, Template, Inventory, Repository, Environment, AccessKey)
- Слой доступа к данным с поддержкой:
  - BoltDB (через sled)
  - SQLite
  - MySQL
  - PostgreSQL
- HTTP API на базе Axum
- CLI на базе Clap
- Система логирования (tracing)
- Документация на русском языке:
  - README.md
  - CONFIG.md
  - API.md
  - MIGRATION.md
- Dockerfile для контейнеризации
- docker-compose.yml для локальной разработки
- Taskfile.yml для автоматизации задач

### Изменено

- Все комментарии и документация переведены на русский язык

### Исправлено

- Начальная версия

---

## [0.0.0] - 2024-01-01

### Примечание

Это начальная версия порта Semaphore UI с Go на Rust.
Многие функции ещё не реализованы или находятся в стадии разработки.

### Известные ограничения

- Не все CRUD-операции реализованы
- Отсутствует поддержка WebSocket
- Планировщик задач (cron) ещё не реализован
- Интеграции с внешними системами требуют доработки
- Раннеры задач ещё не реализованы

---

## Ссылки

- [Не выпущено]: https://github.com/semaphoreui/semaphore/compare/v0.0.0...HEAD
- [0.0.0]: https://github.com/semaphoreui/semaphore/releases/tag/v0.0.0
