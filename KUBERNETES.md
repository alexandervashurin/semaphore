# Kubernetes Integration в Velum

> **Интеграция с Kubernetes для запуска задач в контейнерах**

## 📖 Оглавление

- [Обзор](#обзор)
- [Возможности](#возможности)
- [Установка и настройка](#установка-и-настройка)
- [Использование](#использование)
- [Kubernetes Job](#kubernetes-job)
- [Helm Integration](#helm-integration)
- [Примеры](#примеры)

---

## 📋 Обзор

Velum поддерживает запуск задач в Kubernetes кластерах через:

- **Kubernetes Jobs** - запуск задач в изолированных контейнерах
- **Helm Charts** - управление приложениями через Helm
- **kubectl integration** - выполнение произвольных kubectl команд

**Преимущества:**

| Преимущество | Описание |
|--------------|----------|
| **Изоляция** | Каждая задача выполняется в отдельном Pod |
| **Масштабируемость** | Использование ресурсов Kubernetes кластера |
| **Гибкость** | Поддержка любых Docker образов |
| **Безопасность** | Использование ServiceAccount и RBAC |

---

## ✨ Возможности

### Kubernetes Jobs

- ✅ Запуск задач в Kubernetes Jobs
- ✅ Мониторинг статуса выполнения
- ✅ Получение логов Pod
- ✅ Настройка ресурсов (CPU, Memory)
- ✅ Переменные окружения
- ✅ Service Account

### Helm Integration

- ✅ Установка Helm charts
- ✅ Обновление release
- ✅ Rollback к предыдущей версии
- ✅ Управление repositories
- ✅ Получение статуса release

### Kubectl Commands

- ✅ Выполнение произвольных kubectl команд
- ✅ Работа с любым namespace
- ✅ Поддержка multiple contexts

---

## 🔧 Установка и настройка

### Требования

- Kubernetes кластер 1.20+
- kubectl настроен и доступен
- Helm 3.0+ (опционально)

### Настройка подключения

```bash
# Проверка подключения
kubectl cluster-info

# Настройка контекста
kubectl config use-context my-cluster

# Проверка прав
kubectl auth can-i create jobs
```

### Конфигурация Semaphore

```json
{
  "kubernetes": {
    "enabled": true,
    "kubeconfig_path": "/home/user/.kube/config",
    "default_namespace": "semaphore",
    "context": "my-cluster",
    "job_config": {
      "default_image": "alpine:latest",
      "cpu_limit": "1000m",
      "memory_limit": "1Gi",
      "ttl_seconds": 3600
    },
    "helm_config": {
      "helm_path": "/usr/local/bin/helm",
      "repositories": [
        {
          "name": "bitnami",
          "url": "https://charts.bitnami.com/bitnami"
        }
      ]
    }
  }
}
```

### Переменные окружения

```bash
# Путь к kubeconfig
export KUBECONFIG=/home/user/.kube/config

# Namespace по умолчанию
export SEMAPHORE_K8S_NAMESPACE=semaphore

# Контекст
export SEMAPHORE_K8S_CONTEXT=my-cluster
```

---

## 💡 Использование

### Запуск Kubernetes Job

```rust
use semaphore::kubernetes::{KubernetesClient, KubernetesJob, JobConfig};

// Создаём клиента
let config = KubernetesConfig::default();
let client = KubernetesClient::new(config)?;

// Проверяем подключение
assert!(client.check_connection()?);

// Конфигурируем Job
let job_config = JobConfig {
    name: "my-task".to_string(),
    namespace: Some("semaphore".to_string()),
    image: "alpine:latest".to_string(),
    command: Some(vec!["/bin/sh".to_string()]),
    args: Some(vec!["-c".to_string(), "echo Hello".to_string()]),
    ..Default::default()
};

// Создаём и запускаем Job
let job = KubernetesJob::new(job_config);
job.run(&client)?;

// Ждём завершения
let status = job.wait_for_completion(&client, 300)?;
println!("Job status: {}", status);

// Получаем логи
let logs = client.get_pod_logs(&format!("{}-{}", job_config.name, pod_suffix), None)?;
println!("Logs: {}", logs);
```

### Работа с Helm

```rust
use semaphore::kubernetes::{HelmClient, HelmChart, HelmRelease};

// Создаём Helm клиент
let helm = HelmClient::new()
    .with_namespace("production".to_string());

// Проверяем helm
helm.check_helm()?;

// Добавляем репозиторий
helm.add_repo("bitnami", "https://charts.bitnami.com/bitnami")?;

// Устанавливаем chart
let chart = HelmChart {
    name: "nginx".to_string(),
    version: Some("1.0.0".to_string()),
    repo: Some("bitnami".to_string()),
    path: None,
};

let release = helm.install("my-nginx", &chart, None, None)?;
println!("Installed release: {}", release.name);

// Получаем статус
let status = helm.get_release_status("my-nginx", None)?;
println!("Release status: {}", status);

// Обновляем release
let mut values = std::collections::HashMap::new();
values.insert("replicaCount".to_string(), "3".to_string());
helm.upgrade("my-nginx", &chart, None, Some(&values))?;

// Rollback при необходимости
helm.rollback("my-nginx", 1, None)?;

// Удаляем release
helm.uninstall("my-nginx", None)?;
```

### Kubectl команды

```rust
use semaphore::kubernetes::KubernetesClient;

let client = KubernetesClient::new(KubernetesConfig::default())?;

// Получаем список namespace
let namespaces = client.list_namespaces()?;
println!("Namespaces: {:?}", namespaces);

// Получаем Pod
let pod_json = client.get_pod("my-pod", Some("default"))?;
println!("Pod: {}", pod_json);

// Получаем логи
let logs = client.get_pod_logs("my-pod", Some("default"))?;
println!("Logs: {}", logs);

// Удаляем Pod
client.delete_pod("my-pod", Some("default"))?;

// Произвольная команда
let output = client.run_command(&[
    "get", "pods", "-n", "default", "-o", "wide"
])?;
println!("Pods: {}", output);
```

---

## 📝 Kubernetes Job

### YAML пример

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: semaphore-task
  namespace: semaphore
  labels:
    app: semaphore
spec:
  ttlSecondsAfterFinished: 3600
  backoffLimit: 3
  activeDeadlineSeconds: 7200
  template:
    spec:
      restartPolicy: Never
      serviceAccountName: semaphore
      containers:
      - name: semaphore-task
        image: alpine:latest
        command: ["/bin/sh"]
        args: ["-c", "echo Hello from Semaphore"]
        resources:
          limits:
            cpu: 1000m
            memory: 1Gi
          requests:
            cpu: 100m
            memory: 128Mi
```

### Конфигурация ресурсов

```rust
let job_config = JobConfig {
    name: "resource-limited-job".to_string(),
    image: "my-app:latest".to_string(),
    cpu_limit: Some("2000m".to_string()),
    memory_limit: Some("2Gi".to_string()),
    cpu_request: Some("500m".to_string()),
    memory_request: Some("512Mi".to_string()),
    ..Default::default()
};
```

### Переменные окружения

```rust
use k8s_openapi::api::core::v1::EnvVar;

let job_config = JobConfig {
    name: "env-job".to_string(),
    image: "my-app:latest".to_string(),
    env: Some(vec![
        EnvVar {
            name: "DATABASE_URL".to_string(),
            value: Some("postgres://localhost:5432".to_string()),
            ..Default::default()
        },
        EnvVar {
            name: "API_KEY".to_string(),
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: "my-secret".to_string(),
                    key: "api-key".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
    ]),
    ..Default::default()
};
```

---

## 📦 Helm Integration

### Поддерживаемые команды

| Команда | Описание |
|---------|----------|
| `install` | Установка chart |
| `upgrade` | Обновление release |
| `uninstall` | Удаление release |
| `status` | Получение статуса |
| `rollback` | Откат к revision |
| `history` | История release |
| `repo add` | Добавление репозитория |
| `repo update` | Обновление репозиториев |

### Пример values.yaml

```yaml
# values.yaml для nginx chart
replicaCount: 3
image:
  repository: nginx
  tag: "1.21"
service:
  type: ClusterIP
  port: 80
resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 100m
    memory: 128Mi
```

### Использование values

```rust
let mut values = std::collections::HashMap::new();
values.insert("replicaCount".to_string(), "3".to_string());
values.insert("image.tag".to_string(), "1.21".to_string());
values.insert("service.type".to_string(), "LoadBalancer".to_string());

helm.install("my-nginx", &chart, None, Some(&values))?;
```

---

## 🔐 Безопасность

### ServiceAccount

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: semaphore
  namespace: semaphore
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: semaphore-role
  namespace: semaphore
rules:
- apiGroups: ["batch"]
  resources: ["jobs"]
  verbs: ["get", "list", "create", "delete"]
- apiGroups: [""]
  resources: ["pods", "pods/log"]
  verbs: ["get", "list", "delete"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: semaphore-binding
  namespace: semaphore
subjects:
- kind: ServiceAccount
  name: semaphore
  namespace: semaphore
roleRef:
  kind: Role
  name: semaphore-role
  apiGroup: rbac.authorization.k8s.io
```

### Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: semaphore-network-policy
  namespace: semaphore
spec:
  podSelector:
    matchLabels:
      app: semaphore
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: semaphore
  egress:
  - to:
    - namespaceSelector: {}
```

---

## 🐛 Troubleshooting

### Проблема: Job не запускается

**Решение:**
```bash
# Проверяем события
kubectl get events -n semaphore --sort-by='.lastTimestamp'

# Проверяем Pod
kubectl get pods -n semaphore -l job-name=my-job

# Описываем Pod
kubectl describe pod my-job-pod -n semaphore
```

### Проблема: Helm не найден

**Решение:**
```bash
# Устанавливаем Helm
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Проверяем версию
helm version
```

### Проблема: Permission denied

**Решение:**
```bash
# Проверяем права
kubectl auth can-i create jobs -n semaphore

# Создаём ServiceAccount и RoleBinding
kubectl apply -f rbac.yaml
```

---

## 🔗 Ссылки

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Helm Documentation](https://helm.sh/docs/)
- [kubectl Cheat Sheet](https://kubernetes.io/docs/reference/kubectl/cheatsheet/)
- [Semaphore Kubernetes Examples](https://github.com/alexandervashurin/semaphore/tree/master/kubernetes)

---

*Последнее обновление: 10 марта 2026 г.*
