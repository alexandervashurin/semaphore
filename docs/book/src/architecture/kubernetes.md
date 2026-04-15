# Интеграция с Kubernetes

> K8s-клиент, Helm, управление Jobs в Velum
>
> 📖 См. также: [Обзор системы](./system-overview.md), [Развёртывание в Kubernetes](../deployment/kubernetes-deployment.md), [WebSocket API](../api-reference/websocket-api.md)

---

## Обзор

Velum включает встроенный клиент Kubernetes для управления ресурсами кластера прямо из веб-интерфейса.

## Поддерживаемые ресурсы

| Категория | Ресурсы |
|-----------|---------|
| **Вычисления** | Pod, Deployment, ReplicaSet, DaemonSet, StatefulSet |
| **Сеть** | Service, Ingress, ConfigMap |
| **Хранение** | PersistentVolume, PersistentVolumeClaim |
| **Безопасность** | Secret, ServiceAccount, Role, RoleBinding, ClusterRole |
| **Автоматизация** | Job, CronJob |
| **Мониторинг** | Events, Metrics |

## Helm

Velum поддерживает установку и управление Helm-чартами:

- Установка чартов из репозиториев
- Обновление релизов
- Откат к предыдущим версиям

## GitOps

Velum поддерживает GitOps-паттерн:

- Синхронизация состояния кластера с Git-репозиторием
- Автоматическое применение манифестов
- Отслеживание расхождений

---

## Следующие шаги

- [Развёртывание в Kubernetes](../deployment/kubernetes-deployment.md) — развёртывание самого Velum
- [Обзор системы](./system-overview.md) — архитектура целиком
- [WebSocket API](../api-reference/websocket-api.md) — потоковые события K8s
