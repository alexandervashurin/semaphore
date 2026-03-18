# 📊 Prometheus Metrics - Документация

> **Мониторинг Velum с помощью Prometheus**

---

## 📋 Содержание

1. [Обзор](#обзор)
2. [Быстрый старт](#быстрый-старт)
3. [Доступные метрики](#доступные-метрики)
4. [API Endpoints](#api-endpoints)
5. [Интеграция с Grafana](#интеграция-с-grafana)
6. [Примеры запросов](#примеры-запросов)

---

## 📖 Обзор

Velum предоставляет встроенную поддержку Prometheus метрик для мониторинга:

- **Задачи**: количество, длительность, статусы
- **Раннеры**: активные раннеры
- **Проекты**: количество проектов, шаблонов, инвентарей
- **Пользователи**: количество пользователей
- **Система**: использование CPU, памяти, uptime

---

## 🚀 Быстрый старт

### 1. Доступ к метрикам

```bash
# Prometheus формат
curl http://localhost:3000/api/metrics

# JSON формат
curl http://localhost:3000/api/metrics/json
```

### 2. Настройка Prometheus

Добавьте в `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'semaphore'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/api/metrics'
    scrape_interval: 15s
```

### 3. Перезапуск Prometheus

```bash
systemctl restart prometheus
```

---

## 📈 Доступные метрики

### Задачи (Tasks)

| Метрика | Тип | Описание |
|---------|-----|----------|
| `semaphore_tasks_total` | Counter | Общее количество задач |
| `semaphore_tasks_success_total` | Counter | Количество успешных задач |
| `semaphore_tasks_failed_total` | Counter | Количество проваленных задач |
| `semaphore_tasks_stopped_total` | Counter | Количество остановленных задач |
| `semaphore_task_duration_seconds` | Histogram | Длительность выполнения задач |
| `semaphore_task_queue_time_seconds` | Histogram | Время ожидания в очереди |
| `semaphore_tasks_running` | Gauge | Количество запущенных задач |
| `semaphore_tasks_queued` | Gauge | Количество задач в очереди |

### Раннеры (Runners)

| Метрика | Тип | Описание |
|---------|-----|----------|
| `semaphore_runners_active` | Gauge | Количество активных раннеров |

### Ресурсы (Resources)

| Метрика | Тип | Описание |
|---------|-----|----------|
| `semaphore_projects_total` | Gauge | Общее количество проектов |
| `semaphore_templates_total` | Gauge | Общее количество шаблонов |
| `semaphore_inventories_total` | Gauge | Общее количество инвентарей |
| `semaphore_repositories_total` | Gauge | Общее количество репозиториев |
| `semaphore_users_total` | Gauge | Общее количество пользователей |

### Система (System)

| Метрика | Тип | Описание |
|---------|-----|----------|
| `semaphore_system_cpu_usage_percent` | Gauge | Использование CPU (%) |
| `semaphore_system_memory_usage_mb` | Gauge | Использование памяти (MB) |
| `semaphore_system_uptime_seconds` | Gauge | Время работы (секунды) |
| `semaphore_system_healthy` | Gauge | Статус здоровья (1/0) |

---

## 🌐 API Endpoints

### GET /api/metrics

Возвращает метрики в формате Prometheus.

**Content-Type:** `text/plain; version=0.0.4`

**Пример ответа:**

```
# HELP semaphore_tasks_total Общее количество задач
# TYPE semaphore_tasks_total counter
semaphore_tasks_total 1500

# HELP semaphore_tasks_success_total Количество успешных задач
# TYPE semaphore_tasks_success_total counter
semaphore_tasks_success_total 1425

# HELP semaphore_task_duration_seconds Длительность выполнения задач
# TYPE semaphore_task_duration_seconds histogram
semaphore_task_duration_seconds_bucket{le="0.5"} 100
semaphore_task_duration_seconds_bucket{le="1.0"} 250
semaphore_task_duration_seconds_bucket{le="5.0"} 800
semaphore_task_duration_seconds_bucket{le="10.0"} 1200
semaphore_task_duration_seconds_bucket{le="+Inf"} 1500
semaphore_task_duration_seconds_sum 45000.5
semaphore_task_duration_seconds_count 1500
```

### GET /api/metrics/json

Возвращает метрики в формате JSON.

**Content-Type:** `application/json`

**Пример ответа:**

```json
{
  "tasks": {
    "total": 1500,
    "success": 1425,
    "failed": 75
  },
  "projects": 10,
  "templates": 50,
  "users": 25
}
```

---

## 📊 Интеграция с Grafana

### 1. Добавление источника данных

1. Откройте Grafana
2. Перейдите в **Configuration** → **Data sources**
3. Нажмите **Add data source**
4. Выберите **Prometheus**
5. Укажите URL: `http://localhost:9090`
6. Нажмите **Save & test**

### 2. Создание дашборда

#### Задачи по статусам

```promql
# Успешные задачи
rate(semaphore_tasks_success_total[5m])

# Проваленные задачи
rate(semaphore_tasks_failed_total[5m])

# Остановленные задачи
rate(semaphore_tasks_stopped_total[5m])
```

#### Длительность задач

```promql
# Средняя длительность
rate(semaphore_task_duration_seconds_sum[5m]) 
/ 
rate(semaphore_task_duration_seconds_count[5m])

# 95-й перцентиль
histogram_quantile(0.95, 
  rate(semaphore_task_duration_seconds_bucket[5m])
)
```

#### Использование ресурсов

```promql
# CPU usage
semaphore_system_cpu_usage_percent

# Memory usage
semaphore_system_memory_usage_mb

# Uptime
semaphore_system_uptime_seconds
```

### 3. Пример дашборда

**Velum Overview:**

- **Panel 1**: Tasks Total (Stat)
- **Panel 2**: Success Rate (Gauge)
- **Panel 3**: Task Duration (Time series)
- **Panel 4**: Active Runners (Stat)
- **Panel 5**: CPU Usage (Time series)
- **Panel 6**: Memory Usage (Time series)

---

## 🔍 Примеры запросов PromQL

### Основные метрики

```promql
# Общее количество задач
sum(semaphore_tasks_total)

# Процент успеха
sum(semaphore_tasks_success_total) / sum(semaphore_tasks_total) * 100

# Задач в минуту
rate(semaphore_tasks_total[1m]) * 60

# Средняя длительность задачи
sum(rate(semaphore_task_duration_seconds_sum[5m])) 
/ 
sum(rate(semaphore_task_duration_seconds_count[5m]))
```

### Alerting правила

```yaml
groups:
  - name: semaphore
    rules:
      # Высокий процент проваленных задач
      - alert: HighTaskFailureRate
        expr: |
          (
            sum(rate(semaphore_tasks_failed_total[5m])) 
            / 
            sum(rate(semaphore_tasks_total[5m]))
          ) * 100 > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Высокий процент проваленных задач"
          description: "Процент проваленных задач: {{ $value }}%"

      # Мало активных раннеров
      - alert: LowActiveRunners
        expr: semaphore_runners_active < 1
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Нет активных раннеров"
          description: "Активных раннеров: {{ $value }}"

      # Высокое использование памяти
      - alert: HighMemoryUsage
        expr: semaphore_system_memory_usage_mb > 1024
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Высокое использование памяти"
          description: "Использование памяти: {{ $value }} MB"
```

---

## 🧪 Тестирование

### Проверка доступности метрик

```bash
# Проверка endpoint
curl -I http://localhost:3000/api/metrics

# Проверка формата
curl http://localhost:3000/api/metrics | head -20

# Проверка JSON
curl http://localhost:3000/api/metrics/json | jq
```

### Prometheus targets

Откройте в браузере: `http://localhost:9090/targets`

Статус должен быть **UP**.

---

## 📚 Ссылки

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [PromQL Basics](https://prometheus.io/docs/prometheus/latest/querying/basics/)

---

*Последнее обновление: 9 марта 2026 г.*
