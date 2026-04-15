# Развёртывание в Kubernetes

> Развёртывание Velum в Kubernetes
>
> 📖 См. также: [Docker](./docker-deployment.md), [Конфигурация](../getting-started/configuration.md), [Продакшен](./production-setup.md), [Интеграция с Kubernetes](../architecture/kubernetes.md)

---

## Быстрое развёртывание

```bash
kubectl apply -f deploy/kubernetes/
```

---

## Helm-чарт

```bash
helm install velum ./deploy/kubernetes/helm/velum \
  --set velum.admin.password=admin123
```

---

## Компоненты

| Ресурс | Имя | Назначение |
|--------|-----|-----------|
| Deployment | `velum-server` | Основное приложение |
| Service | `velum-service` | Внутренний доступ в кластере |
| Ingress | `velum-ingress` | Внешний HTTP-доступ |
| ConfigMap | `velum-config` | Конфигурация |
| Secret | `velum-secrets` | Конфиденциальные данные |
| PVC | `velum-data` | Постоянное хранилище |

---

## Kubernetes UI

Velum включает **33 страницы управления Kubernetes**:

- Поды, Deployment, ReplicaSet, DaemonSet, StatefulSet
- Сервисы, ConfigMap, Secret, Ingress
- RBAC, Helm, Jobs, CronJob
- Метрики, события, аудит-логи
- Backup/Restore, GitOps, устранение проблем

Доступ по адресу: `http://velum-url/kubernetes`

См. также: [Интеграция с Kubernetes](../architecture/kubernetes.md)

---

## Масштабирование

```bash
kubectl scale deployment velum-server --replicas=3
```

С режимом HA через Redis несколько экземпляров разделяют состояние через Redis pub/sub.

См. также: [Продакшен](./production-setup.md)

---

## Следующие шаги

- [Docker](./docker-deployment.md) — варианты Docker Compose
- [Продакшен](./production-setup.md) — усиление для продакшена
- [Интеграция с Kubernetes](../architecture/kubernetes.md) — детали K8s API
