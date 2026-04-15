# Документация Velum

> **Velum** — open-source платформа для автоматизации DevOps (Rust + Kubernetes UI)
>
> Управляйте плейбуками, шаблонами, задачами и инфраструктурой как кодом.

---

## Быстрые ссылки

| Ресурс | Описание |
|--------|----------|
| [Быстрый старт](./getting-started/quick-start.md) | Установка и запуск Velum за 5 минут |
| [REST API](./api-reference/rest-api.md) | Полная справка по REST API |
| [OpenAPI](./api-reference/openapi.md) | Интерактивная документация по API (ReDoc) |
| [Rust API](./development/rust-api.md) | Автоматически сгенерированная документация Rust API |
| [Docker](./deployment/docker-deployment.md) | Развёртывание через Docker Compose |
| [Kubernetes](./deployment/kubernetes-deployment.md) | Руководство по интеграции с K8s |

## Статистика проекта

| Метрика | Значение |
|---------|----------|
| **Язык** | Rust (бэкенд) + Vanilla JS (фронтенд) |
| **Тесты** | 6550+ юнит-тестов, ~85% покрытие |
| **API эндпоинты** | 135+ REST + GraphQL + WebSocket |
| **Размер Docker** | ~23 МБ (оптимизированная сборка) |
| **Платформы** | Linux amd64/arm64, macOS amd64/arm64 |

## Внешние ссылки

- [Репозиторий GitHub](https://github.com/alexandervashurin/semaphore)
- [Баг-трекер](https://github.com/alexandervashurin/semaphore/issues)
- [Список изменений](./resources/changelog.md)
- [Спецификация OpenAPI](../api-docs.yml)
- [Коллекция Postman](../.postman/)

## Разделы документации

- 🚀 [Начало работы](./getting-started/quick-start.md) — установка и настройка
- 🏗️ [Развёртывание](./deployment/docker-deployment.md) — Docker и Kubernetes
- 🔌 [API](./api-reference/rest-api.md) — REST, GraphQL, WebSocket, MCP
- 🏛️ [Архитектура](./architecture/system-overview.md) — устройство системы
- 🛠️ [Разработка](./development/dev-setup.md) — настройка окружения
- 🧩 [Расширения](./extensions/vscode.md) — VS Code, Terraform, плагины
- 🐛 [Устранение проблем](./troubleshooting/common-issues.md) — FAQ и отладка
- 🤝 [Участие](./contributing/contributing.md) — как внести вклад
