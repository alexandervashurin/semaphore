# Расширение VS Code

> IntelliSense, сниппеты и API-интеграция для Velum
>
> 📖 См. также: [Terraform](./terraform.md), [Система плагинов](./plugins.md), [Настройка окружения](../development/dev-setup.md)

---

## Возможности

### IntelliSense для шаблонов задач

Автодополнение для файлов Ansible и Terraform:
- Названия шаблонов с сервера Velum
- Ключи задач Ansible (`name`, `hosts`, `become`, `tasks` и т.д.)
- Модули Ansible (`command`, `copy`, `service`, `k8s` и т.д.)
- Блоки Terraform (`resource`, `variable`, `provider` и т.д.)

### Сниппеты плейбуков

| Префикс | Описание |
|---------|----------|
| `velum-playbook` | Шаблон полного плейбука Ansible |
| `velum-task` | Отдельная задача Ansible |
| `velum-role` | Подключение роли Ansible |
| `velum-handler` | Определение обработчика |
| `velum-tf-provider` | Блок провайдера Terraform |
| `velum-tf-resource` | Блок ресурса Terraform |
| `velum-tf-variable` | Переменная Terraform с валидацией |
| `velum-params` | Документация параметров Velum |

### Команды

| Команда | Описание |
|---------|----------|
| `Velum: Login to Velum Server` | Настройка URL сервера и API-токена |
| `Velum: List Projects` | Обзор и выбор проекта |
| `Velum: List Templates` | Просмотр шаблонов проекта |
| `Velum: Run Task from Template` | Запуск задачи из шаблона |
| `Velum: View Task Logs` | Просмотр логов задачи |

---

## Установка

1. Склонируйте репозиторий
2. Откройте `vscode-extension/` в VS Code
3. Установите зависимости: `npm install`
4. Нажмите `F5` для запуска Extension Development Host

---

## Конфигурация

| Настройка | По умолчанию | Описание |
|-----------|-------------|----------|
| `velum.serverUrl` | `http://localhost:3000` | URL сервера Velum |
| `velum.apiToken` | *(пусто)* | Ваш API-токен |
| `velum.projectId` | *(нет)* | ID проекта по умолчанию |

---

## Следующие шаги

- [Terraform](./terraform.md) — управление Velum через Terraform
- [Система плагинов](./plugins.md) — написание своих плагинов
- [Настройка окружения](../development/dev-setup.md) — локальное окружение
