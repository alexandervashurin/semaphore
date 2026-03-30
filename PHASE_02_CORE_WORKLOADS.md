# 📘 ФАЗА 2: Core Workloads (Недели 3-5)

> **Цель:** Реализовать полный CRUD для основных workload ресурсов: Pods, Deployments, ReplicaSets, DaemonSets, StatefulSets.

---

## 📋 Задачи Фазы 2

### 2.1. Pods API (Неделя 3)

**Файлы:**
```
rust/src/api/handlers/kubernetes/
├── pods.rs             # Новый файл
├── pods_logs.rs        # Новый файл
└── pods_exec.rs        # Новый файл
```

**API Endpoints:**
```
GET    /api/kubernetes/pods                                    # Список всех pod'ов
GET    /api/kubernetes/namespaces/{ns}/pods                    # Список pod'ов в namespace
GET    /api/kubernetes/namespaces/{ns}/pods/{name}             # Детали pod
DELETE /api/kubernetes/namespaces/{ns}/pods/{name}             # Удалить pod
POST   /api/kubernetes/namespaces/{ns}/pods/{name}/evict       # Evict pod
GET    /api/kubernetes/namespaces/{ns}/pods/{name}/logs        # Логи
WS     /api/kubernetes/namespaces/{ns}/pods/{name}/logs/stream # WebSocket stream логов
WS     /api/kubernetes/namespaces/{ns}/pods/{name}/exec        # WebSocket exec terminal
GET    /api/kubernetes/namespaces/{ns}/pods/{name}/yaml        # YAML manifest
PUT    /api/kubernetes/namespaces/{ns}/pods/{name}/yaml        # Обновить YAML
```

**Ключевые функции:**
- [ ] List pods с фильтрами (label selector, field selector)
- [ ] Get pod details с полной информацией
- [ ] Delete pod с graceful period
- [ ] Evict pod для drain node операций
- [ ] Get logs с tail lines и follow режимом
- [ ] WebSocket stream для логов
- [ ] WebSocket exec terminal (stdin/stdout/stderr)
- [ ] YAML export/import

---

### 2.2. Deployments API (Неделя 3)

**Файлы:**
```
rust/src/api/handlers/kubernetes/deployments.rs
```

**API Endpoints:**
```
GET    /api/kubernetes/deployments                             # Список всех deployment'ов
GET    /api/kubernetes/namespaces/{ns}/deployments/{name}      # Детали deployment
POST   /api/kubernetes/deployments                             # Создать deployment
PUT    /api/kubernetes/namespaces/{ns}/deployments/{name}      # Обновить deployment
DELETE /api/kubernetes/namespaces/{ns}/deployments/{name}      # Удалить deployment
POST   /api/kubernetes/namespaces/{ns}/deployments/{name}/scale # Scale deployment
POST   /api/kubernetes/namespaces/{ns}/deployments/{name}/restart # Restart rollout
POST   /api/kubernetes/namespaces/{ns}/deployments/{name}/pause   # Pause rollout
POST   /api/kubernetes/namespaces/{ns}/deployments/{name}/resume  # Resume rollout
POST   /api/kubernetes/namespaces/{ns}/deployments/{name}/rollback # Rollback к ревизии
GET    /api/kubernetes/namespaces/{ns}/deployments/{name}/history # История rollout
GET    /api/kubernetes/namespaces/{ns}/deployments/{name}/replicasets # Linked ReplicaSets
```

**Ключевые функции:**
- [ ] Scale deployment (изменение количества реплик)
- [ ] Restart rollout (через annotation rollout.kubernetes.io/restartedAt)
- [ ] Pause/resume rollout
- [ ] Rollback к предыдущей ревизии
- [ ] История rollout с деталями
- [ ] Получение связанных ReplicaSets

**Пример кода (scale deployment):**
```rust
/// Scale deployment
/// POST /api/kubernetes/namespaces/{ns}/deployments/{name}/scale
pub async fn scale_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Deployment> = client.api::<Deployment>(Some(&namespace));

    let replicas = payload
        .get("replicas")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| Error::ValidationError("replicas is required".to_string()))?;

    let mut deployment = api
        .get(&name)
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    deployment.spec.as_mut().unwrap().replicas = Some(replicas as i32);

    let updated = api
        .replace(&name, &Default::default(), &deployment)
        .await
        .map_err(|e| Error::KubernetesError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "replicas": replicas,
        "deployment": name
    })))
}
```

---

### 2.3. ReplicaSets API (Неделя 4)

**Файлы:**
```
rust/src/api/handlers/kubernetes/replicasets.rs
```

**API Endpoints:**
```
GET    /api/kubernetes/replicasets                             # Список всех ReplicaSets
GET    /api/kubernetes/namespaces/{ns}/replicasets/{name}      # Детали ReplicaSet
DELETE /api/kubernetes/namespaces/{ns}/replicasets/{name}      # Удалить ReplicaSet
GET    /api/kubernetes/namespaces/{ns}/replicasets/{name}/pods # Linked Pods
```

---

### 2.4. DaemonSets API (Неделя 4)

**Файлы:**
```
rust/src/api/handlers/kubernetes/daemonsets.rs
```

**API Endpoints:**
```
GET    /api/kubernetes/daemonsets                              # Список всех DaemonSets
GET    /api/kubernetes/namespaces/{ns}/daemonsets/{name}       # Детали DaemonSet
POST   /api/kubernetes/daemonsets                              # Создать DaemonSet
PUT    /api/kubernetes/namespaces/{ns}/daemonsets/{name}       # Обновить DaemonSet
DELETE /api/kubernetes/namespaces/{ns}/daemonsets/{name}       # Удалить DaemonSet
GET    /api/kubernetes/namespaces/{ns}/daemonsets/{name}/pods  # Linked Pods
```

---

### 2.5. StatefulSets API (Неделя 4)

**Файлы:**
```
rust/src/api/handlers/kubernetes/statefulsets.rs
```

**API Endpoints:**
```
GET    /api/kubernetes/statefulsets                            # Список всех StatefulSets
GET    /api/kubernetes/namespaces/{ns}/statefulsets/{name}     # Детали StatefulSet
POST   /api/kubernetes/statefulsets                            # Создать StatefulSet
PUT    /api/kubernetes/namespaces/{ns}/statefulsets/{name}     # Обновить StatefulSet
DELETE /api/kubernetes/namespaces/{ns}/statefulsets/{name}     # Удалить StatefulSet
POST   /api/kubernetes/namespaces/{ns}/statefulsets/{name}/scale # Scale StatefulSet
GET    /api/kubernetes/namespaces/{ns}/statefulsets/{name}/pods # Linked Pods (ordered)
```

---

### 2.6. Frontend — Pods Page

**Файлы:**
```
web/kubernetes/pages/k8s-pods.html
web/kubernetes/pages/pods.js
web/kubernetes/components/pod-table.js
web/kubernetes/components/pod-details.js
web/kubernetes/components/logs-viewer.js
web/kubernetes/components/terminal-exec.js
```

**Функционал Pods Page:**
- [ ] Таблица pod'ов с фильтрами по namespace
- [ ] Статусы цветными бейджами (Running/Pending/Failed/CrashLoopBackOff)
- [ ] Быстрые действия: View, Logs, Terminal, Delete
- [ ] Детальная страница pod с:
  - Containers list со статусами
  - Volumes mounts
  - Environment variables
  - Events pod'а
  - Графики CPU/Memory (из metrics API)
- [ ] YAML editor modal
- [ ] WebSocket terminal для exec

**Пример Pod Table Component:**
```javascript
class PodTable {
  constructor(containerId, options = {}) {
    this.container = document.getElementById(containerId);
    this.namespace = options.namespace || null;
    this.onSelect = options.onSelect || (() => {});
    this.pods = [];

    this.render();
    this.loadPods();
  }

  async loadPods() {
    try {
      this.pods = this.namespace
        ? await k8s.listPods(this.namespace)
        : await k8s.listPods();
      this.render();
    } catch (error) {
      console.error('Failed to load pods:', error);
      this.container.innerHTML = `
        <div class="error-message">
          <i class="fa-solid fa-triangle-exclamation"></i>
          Failed to load pods
        </div>
      `;
    }
  }

  render() {
    if (this.pods.length === 0) {
      this.container.innerHTML = `
        <div class="k8s-empty">
          <i class="fa-solid fa-box"></i>
          <p>No pods found</p>
        </div>
      `;
      return;
    }

    this.container.innerHTML = `
      <table class="k8s-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Namespace</th>
            <th>Status</th>
            <th>Containers</th>
            <th>Node</th>
            <th>IP</th>
            <th>Restarts</th>
            <th>Age</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          ${this.pods.map(pod => `
            <tr class="k8s-table-row" data-pod-name="${pod.name}">
              <td>
                <a href="#" onclick="viewPod('${pod.namespace}', '${pod.name}')">
                  <i class="fa-solid fa-box"></i> ${pod.name}
                </a>
              </td>
              <td>${pod.namespace}</td>
              <td>
                <span class="badge badge-${this.getStatusClass(pod.phase)}">
                  ${pod.phase}
                </span>
              </td>
              <td>${pod.containers}</td>
              <td>${pod.node_name}</td>
              <td>${pod.pod_ip}</td>
              <td>${pod.restart_count}</td>
              <td>${this.formatAge(pod.created_at)}</td>
              <td class="actions">
                <button class="btn-icon" onclick="viewPodLogs('${pod.namespace}', '${pod.name}')" title="Logs">
                  <i class="fa-solid fa-file-lines"></i>
                </button>
                <button class="btn-icon" onclick="openPodTerminal('${pod.namespace}', '${pod.name}')" title="Terminal">
                  <i class="fa-solid fa-terminal"></i>
                </button>
                <button class="btn-icon" onclick="deletePod('${pod.namespace}', '${pod.name}')" title="Delete">
                  <i class="fa-solid fa-trash"></i>
                </button>
              </td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    `;
  }

  getStatusClass(phase) {
    switch (phase.toLowerCase()) {
      case 'running': return 'success';
      case 'pending': return 'warning';
      case 'failed': return 'danger';
      case 'succeeded': return 'success';
      default: return 'secondary';
    }
  }

  formatAge(dateString) {
    const date = new Date(dateString);
    const now = new Date();
    const diff = now - date;
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));

    if (days > 0) return `${days}d${hours}h`;
    return `${hours}h`;
  }
}
```

---

### 2.7. Frontend — Deployments Page

**Файлы:**
```
web/kubernetes/pages/k8s-deployments.html
web/kubernetes/pages/deployments.js
web/kubernetes/components/deployment-table.js
web/kubernetes/components/deployment-details.js
web/kubernetes/components/deployment-scaler.js
```

**Функционал Deployments Page:**
- [ ] Таблица deployment'ов
- [ ] Кнопки Scale (+/-)
- [ ] Кнопка Restart
- [ ] Кнопка Rollback с выбором ревизии
- [ ] Визуализация rollout status
- [ ] Связанные ReplicaSets
- [ ] YAML editor

**Пример Deployment Scaler Component:**
```javascript
class DeploymentScaler {
  constructor(deploymentName, namespace, currentReplicas, onChange) {
    this.deploymentName = deploymentName;
    this.namespace = namespace;
    this.replicas = currentReplicas;
    this.onChange = onChange;

    this.render();
  }

  render() {
    return `
      <div class="scaler-control">
        <button class="btn btn-sm btn-secondary" onclick="scaler.scale(-1)">
          <i class="fa-solid fa-minus"></i>
        </button>
        <span class="replicas-display">${this.replicas} replicas</span>
        <button class="btn btn-sm btn-secondary" onclick="scaler.scale(1)">
          <i class="fa-solid fa-plus"></i>
        </button>
      </div>
    `;
  }

  async scale(delta) {
    const newReplicas = Math.max(0, this.replicas + delta);

    try {
      await api.post(
        `/api/kubernetes/namespaces/${this.namespace}/deployments/${this.deploymentName}/scale`,
        { replicas: newReplicas }
      );
      this.replicas = newReplicas;
      this.render();
      this.onChange(newReplicas);
    } catch (error) {
      console.error('Failed to scale deployment:', error);
      alert('Failed to scale deployment');
    }
  }
}
```

---

### 2.8. Logs Viewer Component

**Файлы:**
```
web/kubernetes/components/logs-viewer.js
```

**Функционал:**
- [ ] Streaming логов через WebSocket
- [ ] Выбор контейнера (если несколько)
- [ ] Tail lines настройка
- [ ] Follow toggle
- [ ] Auto-scroll
- [ ] Search/highlight
- [ ] Download logs

**Пример кода:**
```javascript
class LogsViewer {
  constructor(containerId, options = {}) {
    this.container = document.getElementById(containerId);
    this.namespace = options.namespace;
    this.podName = options.podName;
    this.container = options.container;
    this.tailLines = options.tailLines || 100;
    this.follow = options.follow ?? true;
    this.ws = null;

    this.connect();
  }

  connect() {
    const wsUrl = `ws://${window.location.host}/api/kubernetes/namespaces/${this.namespace}/pods/${this.podName}/logs/stream`;
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      console.log('Logs WebSocket connected');
      this.ws.send(JSON.stringify({
        action: 'subscribe',
        container: this.container,
        tailLines: this.tailLines,
        follow: this.follow
      }));
    };

    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      this.appendLog(message.line);
    };

    this.ws.onerror = (error) => {
      console.error('Logs WebSocket error:', error);
    };

    this.ws.onclose = () => {
      console.log('Logs WebSocket closed');
      setTimeout(() => this.connect(), 5000); // Reconnect
    };
  }

  appendLog(line) {
    const logContainer = this.container.querySelector('.logs-content');
    const lineElement = document.createElement('div');
    lineElement.className = 'log-line';
    lineElement.textContent = line;
    logContainer.appendChild(lineElement);

    // Auto-scroll
    if (this.follow) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}
```

---

### 2.9. Terminal Exec Component

**Файлы:**
```
web/kubernetes/components/terminal-exec.js
```

**Функционал:**
- [ ] WebSocket terminal для exec в pod
- [ ] Выбор контейнера
- [ ] Выбор shell (bash/sh)
- [ ] Full-screen режим
- [ ] Copy/paste поддержка

**Пример кода:**
```javascript
class PodTerminal {
  constructor(containerId, options = {}) {
    this.container = document.getElementById(containerId);
    this.namespace = options.namespace;
    this.podName = options.podName;
    this.container = options.container;
    this.command = options.command || ['/bin/sh'];
    this.ws = null;
    this.term = null;

    this.init();
  }

  init() {
    // Создаём xterm.js terminal
    this.term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Monaco, Consolas, monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#ffffff'
      }
    });

    this.term.open(this.container);

    // Подключаемся к WebSocket
    const wsUrl = `ws://${window.location.host}/api/kubernetes/namespaces/${this.namespace}/pods/${this.podName}/exec`;
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      console.log('Terminal WebSocket connected');
      this.ws.send(JSON.stringify({
        action: 'exec',
        container: this.container,
        command: this.command
      }));
    };

    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      if (message.type === 'stdout') {
        this.term.write(message.data);
      }
    };

    this.ws.onclose = () => {
      console.log('Terminal WebSocket closed');
    };

    // Отправляем ввод с терминала
    this.term.onData(data => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({
          type: 'stdin',
          data: data
        }));
      }
    });

    // Handle resize
    this.term.onResize(({ cols, rows }) => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({
          type: 'resize',
          cols,
          rows
        }));
      }
    });
  }

  close() {
    if (this.ws) {
      this.ws.close();
    }
    if (this.term) {
      this.term.dispose();
    }
  }
}
```

---

## ✅ Критерии приемки Фазы 2

- [ ] Pods CRUD работает полностью
- [ ] Logs streaming через WebSocket
- [ ] Exec terminal работает
- [ ] Deployments scale/restart/rollback работают
- [ ] ReplicaSets отображаются
- [ ] DaemonSets отображаются
- [ ] StatefulSets отображаются
- [ ] YAML editor для всех ресурсов
- [ ] Frontend страницы для всех workload типов
- [ ] Покрытие тестами > 80%

---

## 📊 Метрики Фазы 2

| Метрика | Целевое значение |
|---------|------------------|
| Время загрузки списка pod'ов | < 500 ms |
| Время старта terminal сессии | < 2 сек |
| Задержка логов (WebSocket) | < 100 ms |
| Время scale deployment | < 1 сек |
