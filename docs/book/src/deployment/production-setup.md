# Продакшен-настройка

> Усиление, мониторинг и масштабирование для продакшен-развёртываний Velum
>
> 📖 См. также: [Docker](./docker-deployment.md), [Kubernetes](./kubernetes-deployment.md), [Конфигурация](../getting-started/configuration.md), [Устранение проблем](../troubleshooting/common-issues.md)

---

## Рекомендации по безопасности

- Используйте HTTPS с TLS-сертификатами
- Храните секреты в Kubernetes Secrets или HashiCorp Vault
- Не используйте пароль `admin123` — задайте стойкий `VELUM_ADMIN_PASSWORD`
- Ограничьте доступ к API через firewall или ingress rules

## Масштабирование

- Используйте Redis для HA-режима (несколько экземпляров Velum)
- Настройте health checks и readiness probes
- Используйте внешний PostgreSQL (не контейнер)

## Мониторинг

- Velum предоставляет `/metrics` эндпоинт в формате Prometheus
- Логи структурированы — используйте `RUST_LOG=velum=info`
- Проверки здоровья: `/healthz`, `/readyz`

## Резервное копирование

Регулярно бэкапьте:
- Базу данных PostgreSQL
- Тома с данными (playbook, ключи)

См. также: [Типовые проблемы](../troubleshooting/common-issues.md)

---

## Следующие шаги

- [Docker](./docker-deployment.md) — варианты Docker Compose
- [Kubernetes](./kubernetes-deployment.md) — развёртывание в K8s
- [Конфигурация](../getting-started/configuration.md) — переменные окружения
