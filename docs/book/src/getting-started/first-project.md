# Первый проект

> Руководство по созданию первого проекта, шаблона и задачи в Velum.
>
> 📖 См. также: [Быстрый старт](./quick-start.md), [Конфигурация](./configuration.md), [REST API](../api-reference/rest-api.md)

---

## Создание проекта

1. Откройте веб-интерфейс Velum
2. Нажмите **«Создать проект»**
3. Укажите имя и описание проекта

## Создание шаблона

1. Откройте раздел **«Шаблоны»** в проекте
2. Нажмите **«Создать шаблон»**
3. Укажите:
   - Имя шаблона
   - Плейбук Ansible или путь к Terraform
   - Инвентарь (список хостов)
   - Ключи доступа (SSH)

## Запуск задачи

1. Откройте нужный шаблон
2. Нажмите **«Запустить»**
3. Наблюдайте за выполнением в реальном времени через WebSocket

## API-способ

```bash
# Создание проекта
curl -X POST http://localhost:3000/api/projects \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <token>' \
  -d '{"name": "Мой проект"}'

# Создание шаблона
curl -X POST http://localhost:3000/api/templates \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <token>' \
  -d '{"project_id": 1, "name": "Деплой", "playbook": "deploy.yml"}'

# Запуск задачи
curl -X POST http://localhost:3000/api/tasks \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <token>' \
  -d '{"template_id": 1}'
```

---

## Следующие шаги

- [REST API](../api-reference/rest-api.md) — полное описание эндпоинтов
- [Docker](../deployment/docker-deployment.md) — развёртывание
- [Выполнение задач](../architecture/task-execution.md) — как работают задачи
