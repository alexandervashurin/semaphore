# Kubernetes Deployment

> Deploy Velum to Kubernetes
>
> 📖 See also: [[Docker Deployment]], [[Configuration]], [[Production Setup]], [[Kubernetes Integration]]

---

## Quick Deploy

```bash
kubectl apply -f deploy/kubernetes/
```

---

## Helm Chart

```bash
helm install velum ./deploy/kubernetes/helm/velum \
  --set velum.admin.password=admin123
```

---

## Components

| Resource | Name | Purpose |
|----------|------|---------|
| Deployment | `velum-server` | Main application |
| Service | `velum-service` | Internal cluster access |
| Ingress | `velum-ingress` | External HTTP access |
| ConfigMap | `velum-config` | Configuration |
| Secret | `velum-secrets` | Sensitive data |
| PVC | `velum-data` | Persistent storage |

---

## Kubernetes UI

Velum includes **33 Kubernetes management pages**:

- Pods, Deployments, ReplicaSets, DaemonSets, StatefulSets
- Services, ConfigMaps, Secrets, Ingress
- RBAC, Helm, Jobs, CronJobs
- Metrics, Events, Audit Logs
- Backup/Restore, GitOps, Troubleshooting

Access at: http://velum-url/kubernetes

---

## Scaling

```bash
kubectl scale deployment velum-server --replicas=3
```

With Redis HA mode, multiple instances share state via Redis pub/sub.

---

## Next Steps

- [[Docker Deployment]] — Docker Compose options
- [[Production Setup]] — harden for production
- [[Kubernetes Integration]] — K8s API details
