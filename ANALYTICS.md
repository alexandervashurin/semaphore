# 📊 Analytics API - Документация

> **Расширенная аналитика и дашборды для мониторинга системы**

---

## 📋 Содержание

1. [Обзор](#обзор)
2. [Модели данных](#модели-данных)
3. [API Endpoints](#api-endpoints)
4. [Примеры использования](#примеры-использования)

---

## 📖 Обзор

Analytics предоставляет детальную статистику и метрики для:

- **Проектов**: общая статистика, успех задач
- **Задач**: длительность, статусы, производительность
- **Пользователей**: активность, вклад
- **Системы**: здоровье, ресурсы, раннеры
- **Временных рядов**: графики, тренды

---

## 🗃️ Модели данных

### ProjectStats

Основная статистика проекта:

| Поле | Тип | Описание |
|------|-----|----------|
| `project_id` | i64 | ID проекта |
| `project_name` | String | Название |
| `total_tasks` | i64 | Всего задач |
| `successful_tasks` | i64 | Успешных |
| `failed_tasks` | i64 | Проваленных |
| `stopped_tasks` | i64 | Остановленных |
| `pending_tasks` | i64 | Ожидающих |
| `running_tasks` | i64 | Запущенных |
| `total_templates` | i64 | Всего шаблонов |
| `total_users` | i64 | Пользователей |
| `success_rate` | f64 | Процент успеха |
| `avg_task_duration_secs` | f64 | Средняя длительность |

### TaskStats

Статистика задач за период:

| Поле | Тип | Описание |
|------|-----|----------|
| `period` | String | Период |
| `total` | i64 | Всего |
| `success` | i64 | Успешных |
| `failed` | i64 | Проваленных |
| `avg_duration_secs` | f64 | Средняя длительность |
| `max_duration_secs` | f64 | Максимальная |
| `min_duration_secs` | f64 | Минимальная |

### PerformanceMetrics

Метрики производительности:

| Поле | Тип | Описание |
|------|-----|----------|
| `avg_queue_time_secs` | f64 | Среднее время в очереди |
| `avg_execution_time_secs` | f64 | Среднее время выполнения |
| `tasks_per_hour` | f64 | Задач в час |
| `tasks_per_day` | f64 | Задач в день |
| `concurrent_tasks_avg` | f64 | Среднее количество параллельных |
| `concurrent_tasks_max` | i64 | Максимум параллельных |

### SystemStatus

Статус системы:

| Поле | Тип | Описание |
|------|-----|----------|
| `healthy` | bool | Здоров ли |
| `version` | String | Версия |
| `uptime_secs` | i64 | Время работы |
| `active_runners` | i64 | Активные раннеры |
| `running_tasks` | i64 | Запущенные задачи |
| `queued_tasks` | i64 | Задачи в очереди |
| `database_status` | String | Статус БД |

### ChartData

Данные для графиков:

| Поле | Тип | Описание |
|------|-----|----------|
| `label` | String | Метка |
| `value` | f64 | Значение |
| `timestamp` | DateTime | Время |

---

## 🌐 API Endpoints

### GET /api/analytics/system

Получить сводные метрики системы.

**Требуемые права**: Администратор

**Ответ**:

```json
{
  "total_projects": 10,
  "total_users": 25,
  "total_tasks": 1500,
  "total_templates": 50,
  "total_runners": 3,
  "active_runners": 2,
  "running_tasks": 5,
  "queued_tasks": 3,
  "success_rate_24h": 95.5,
  "avg_task_duration_24h": 120.5,
  "tasks_24h": 150,
  "tasks_7d": 800,
  "tasks_30d": 1500
}
```

---

### GET /api/analytics/project/:id

Получить аналитику проекта.

**Требуемые права**: Доступ к проекту

**Параметры**:

| Параметр | Тип | Описание |
|----------|-----|----------|
| `date_from` | DateTime | Начало периода |
| `date_to` | DateTime | Окончание периода |
| `period` | string | Период (hour, day, week, month) |

**Ответ**:

```json
{
  "stats": {
    "project_id": 1,
    "project_name": "Production",
    "total_tasks": 500,
    "successful_tasks": 475,
    "failed_tasks": 20,
    "success_rate": 95.0,
    "avg_task_duration_secs": 180.5
  },
  "task_stats": {
    "period": "day",
    "total": 50,
    "success": 48,
    "failed": 2
  },
  "top_users": [
    {"user_id": 1, "username": "admin", "tasks_count": 100, "success_rate": 98.0}
  ],
  "top_templates": [
    {"id": 1, "name": "Deploy", "value": 200, "type": "template"}
  ]
}
```

---

### GET /api/analytics/project/:id/tasks

Получить статистику задач проекта.

**Параметры**:

| Параметр | Тип | Описание |
|----------|-----|----------|
| `date_from` | DateTime | Начало периода |
| `date_to` | DateTime | Окончание периода |
| `group_by` | string | Группировка (user, template, status) |

**Ответ**:

```json
{
  "period": "2026-03-01/2026-03-31",
  "total": 500,
  "success": 475,
  "failed": 20,
  "stopped": 5,
  "avg_duration_secs": 180.5,
  "max_duration_secs": 3600.0,
  "min_duration_secs": 10.0
}
```

---

### GET /api/analytics/project/:id/users

Получить активность пользователей.

**Ответ**:

```json
[
  {
    "user_id": 1,
    "username": "admin",
    "total_actions": 500,
    "tasks_created": 200,
    "tasks_run": 150,
    "templates_created": 50,
    "last_activity": "2026-03-09T10:00:00Z"
  }
]
```

---

### GET /api/analytics/project/:id/performance

Получить метрики производительности.

**Ответ**:

```json
{
  "avg_queue_time_secs": 5.2,
  "avg_execution_time_secs": 180.5,
  "tasks_per_hour": 15.5,
  "tasks_per_day": 350.0,
  "concurrent_tasks_avg": 2.5,
  "concurrent_tasks_max": 10,
  "resource_usage": {
    "cpu_usage_percent": 45.5,
    "memory_usage_mb": 512.0,
    "disk_usage_mb": 1024.0
  }
}
```

---

### GET /api/analytics/project/:id/chart

Получить данные для графика.

**Параметры**:

| Параметр | Тип | Описание |
|----------|-----|----------|
| `metric` | string | Метрика (tasks, success, failed, duration) |
| `period` | string | Период (hour, day, week, month) |
| `date_from` | DateTime | Начало периода |
| `date_to` | DateTime | Окончание периода |

**Ответ**:

```json
{
  "metric": "tasks",
  "data": [
    {"label": "2026-03-01", "value": 50, "timestamp": "2026-03-01T00:00:00Z"},
    {"label": "2026-03-02", "value": 45, "timestamp": "2026-03-02T00:00:00Z"},
    {"label": "2026-03-03", "value": 60, "timestamp": "2026-03-03T00:00:00Z"}
  ]
}
```

---

### GET /api/analytics/project/:id/slow-tasks

Получить самые медленные задачи.

**Параметры**:

| Параметр | Тип | Описание |
|----------|-----|----------|
| `limit` | i64 | Лимит (по умолчанию 10) |

**Ответ**:

```json
[
  {
    "task_id": 100,
    "task_name": "Deploy Production",
    "template_name": "Deploy",
    "duration_secs": 3600.0,
    "status": "success",
    "created": "2026-03-09T08:00:00Z"
  }
]
```

---

### GET /api/analytics/runners

Получить метрики раннеров.

**Требуемые права**: Администратор

**Ответ**:

```json
[
  {
    "runner_id": 1,
    "runner_name": "runner-1",
    "active": true,
    "tasks_completed": 500,
    "tasks_failed": 10,
    "avg_execution_time_secs": 150.0,
    "last_seen": "2026-03-09T10:00:00Z",
    "uptime_secs": 86400
  }
]
```

---

### GET /api/analytics/health

Получить статус здоровья системы.

**Ответ**:

```json
{
  "healthy": true,
  "version": "0.1.0",
  "uptime_secs": 86400,
  "active_runners": 2,
  "running_tasks": 5,
  "queued_tasks": 3,
  "database_status": "connected",
  "last_check": "2026-03-09T10:00:00Z"
}
```

---

## 💡 Примеры использования

### Получение статистики проекта

```bash
curl -X GET "http://localhost:3000/api/analytics/project/1" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Получение данных для графика

```bash
# Задачи по дням за месяц
curl -X GET "http://localhost:3000/api/analytics/project/1/chart?metric=tasks&period=day&date_from=2026-03-01T00:00:00Z&date_to=2026-03-31T23:59:59Z" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Получение топ пользователей

```bash
curl -X GET "http://localhost:3000/api/analytics/project/1/users?limit=10" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Получение системных метрик

```bash
curl -X GET "http://localhost:3000/api/analytics/system" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Получение статуса здоровья

```bash
curl -X GET "http://localhost:3000/api/analytics/health" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

## 📊 Использование в дашбордах

### Chart.js пример

```javascript
// Получение данных
fetch('/api/analytics/project/1/chart?metric=tasks&period=day')
  .then(r => r.json())
  .then(data => {
    const ctx = document.getElementById('tasksChart').getContext('2d');
    new Chart(ctx, {
      type: 'line',
      data: {
        labels: data.data.map(d => d.label),
        datasets: [{
          label: 'Задачи',
          data: data.data.map(d => d.value),
          borderColor: 'rgb(75, 192, 192)',
          tension: 0.1
        }]
      }
    });
  });
```

### Vue.js пример

```vue
<template>
  <div>
    <h3>Статистика проекта</h3>
    <v-card>
      <v-card-text>
        <div>Всего задач: {{ stats.total_tasks }}</div>
        <div>Успех: {{ stats.success_rate }}%</div>
        <div>Среднее время: {{ stats.avg_task_duration_secs }}с</div>
      </v-card-text>
    </v-card>
  </div>
</template>

<script>
export default {
  data: () => ({
    stats: null
  }),
  async mounted() {
    const response = await fetch('/api/analytics/project/1');
    this.stats = (await response.json()).stats;
  }
}
</script>
```

---

## 🔧 Вычисляемые метрики

### Success Rate

```
success_rate = (successful_tasks / total_tasks) * 100
```

### Average Duration

```
avg_duration = sum(task_durations) / count(completed_tasks)
```

### Tasks Per Hour

```
tasks_per_hour = total_tasks / hours_in_period
```

---

*Последнее обновление: 9 марта 2026 г.*
