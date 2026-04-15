# Провайдер Terraform

> Управление ресурсами Velum через Terraform
>
> 📖 См. также: [VS Code](./vscode.md), [Конфигурация](../getting-started/configuration.md), [Docker](../deployment/docker-deployment.md)

---

## Ресурсы

| Ресурс | Описание |
|--------|----------|
| `velum_project` | Управление проектами Velum |
| `velum_template` | Управление шаблонами задач |
| `velum_access_key` | Управление ключами доступа |

## Источники данных (Data Sources)

| Источник | Описание |
|----------|----------|
| `velum_project` | Поиск проекта по имени |
| `velum_template` | Поиск шаблона по имени |

---

## Пример использования (HCL)

```hcl
provider "velum" {
  server_url = "http://localhost:3000"
  api_token  = var.velum_api_token
}

resource "velum_project" "my_project" {
  name = "Мой проект"
  description = "Проект, управляемый через Terraform"
}

resource "velum_template" "deploy" {
  project_id = velum_project.my_project.id
  name       = "Деплой"
  playbook   = "deploy.yml"
}
```

---

## Разработка

```bash
cd terraform-provider
# Сборка
go build -o terraform-provider-velum

# Тесты
go test ./...
```

---

## Следующие шаги

- [VS Code](./vscode.md) — расширение VS Code
- [Конфигурация](../getting-started/configuration.md) — переменные окружения
- [Docker](../deployment/docker-deployment.md) — развёртывание
