# 🎨 Frontend Development Plan

> **Полноценная реализация frontend с интеграцией всех API endpoints**

---

## 📊 Текущее состояние frontend

### Технологии

| Компонент | Версия | Статус |
|-----------|--------|--------|
| **Vue.js** | 2.6.14 | ✅ Актуально |
| **Vuetify** | 2.6.10 | ✅ Актуально |
| **Vue Router** | 3.5.4 | ✅ Актуально |
| **Axios** | 1.13.5 | ✅ Актуально |
| **Chart.js** | 3.8.0 | ✅ Актуально |
| **Vue I18n** | 8.18.2 | ✅ Актуально |

### Структура проекта

```
web/
├── src/
│   ├── components/        # UI компоненты (60+ файлов)
│   ├── views/             # Страницы приложения
│   │   ├── project/       # Страницы проектов (15 файлов)
│   │   └── ...
│   ├── router/            # Маршрутизация
│   ├── lib/               # Библиотеки
│   │   └── api.js         # API клиент (ТРЕБУЕТ РАСШИРЕНИЯ)
│   ├── plugins/           # Vue плагины
│   └── lang/              # Локализация
└── public/                # Статические файлы
```

### Существующие страницы

| Страница | Маршрут | Статус |
|----------|---------|--------|
| **Аутентификация** | `/auth/login` | ✅ Готово |
| **Проекты** | `/project/:id` | ✅ Готово |
| **История задач** | `/project/:id/history` | ✅ Готово |
| **Шаблоны** | `/project/:id/templates` | ✅ Готово |
| **Инвентари** | `/project/:id/inventory` | ✅ Готово |
| **Репозитории** | `/project/:id/repositories` | ✅ Готово |
| **Окружения** | `/project/:id/environment` | ✅ Готово |
| **Ключи** | `/project/:id/keys` | ✅ Готово |
| **Команда** | `/project/:id/team` | ✅ Готово |
| **Расписание** | `/project/:id/schedule` | ✅ Готово |
| **Интеграции** | `/project/:id/integrations` | ✅ Готово |
| **Настройки** | `/project/:id/settings` | ✅ Готово |
| **Пользователи** | `/users` | ✅ Готово |
| **Раннеры** | `/runners` | ✅ Готово |
| **Задачи** | `/tasks` | ✅ Готово |
| **Приложения** | `/apps` | ✅ Готово |
| **Токены** | `/tokens` | ✅ Готово |

---

## 🔴 Отсутствующий функционал

### 1. Audit Log Interface

**API Endpoints:**
- `GET /api/audit-log` — Поиск записей
- `GET /api/audit-log/:id` — Детали записи
- `GET /api/project/:id/audit-log` — Audit log проекта
- `DELETE /api/audit-log/clear` — Очистка
- `DELETE /api/audit-log/expiry` — Удаление старых

**Необходимые компоненты:**
- `AuditLogList.vue` — Список записей
- `AuditLogDetails.vue` — Детали записи
- `AuditLogFilter.vue` — Фильтры
- `AuditLogChart.vue` — Графики активности

**Страницы:**
```
/project/:id/audit-log — Audit Log проекта
/admin/audit-log — Глобальный Audit Log
```

---

### 2. Webhook Management

**API Endpoints:**
- `GET /api/webhooks` — Список webhook
- `POST /api/webhooks` — Создание
- `PUT /api/webhooks/:id` — Обновление
- `DELETE /api/webhooks/:id` — Удаление
- `POST /api/webhooks/:id/test` — Тест

**Необходимые компоненты:**
- `WebhookList.vue` — Список webhook
- `WebhookForm.vue` — Форма создания/редактирования
- `WebhookTypeSelector.vue` — Выбор типа
- `WebhookTestDialog.vue` — Тестирование
- `WebhookLogView.vue` — История отправок

**Страницы:**
```
/project/:id/webhooks — Управление webhook
```

---

### 3. Analytics Dashboard

**API Endpoints:**
- `GET /api/analytics/system` — Системные метрики
- `GET /api/analytics/project/:id` — Аналитика проекта
- `GET /api/analytics/project/:id/tasks` — Статистика задач
- `GET /api/analytics/project/:id/users` — Активность пользователей
- `GET /api/analytics/project/:id/performance` — Производительность
- `GET /api/analytics/project/:id/chart` — Данные для графиков
- `GET /api/analytics/project/:id/slow-tasks` — Медленные задачи
- `GET /api/analytics/runners` — Метрики раннеров
- `GET /api/analytics/health` — Статус здоровья

**Необходимые компоненты:**
- `AnalyticsDashboard.vue` — Главная панель
- `ProjectStatsCard.vue` — Карточка статистики проекта
- `TaskMetricsChart.vue` — Графики задач
- `UserActivityChart.vue` — Активность пользователей
- `PerformanceMetrics.vue` — Метрики производительности
- `SlowTasksTable.vue` — Таблица медленных задач
- `SystemHealthCard.vue` — Статус здоровья

**Страницы:**
```
/project/:id/analytics — Аналитика проекта
/admin/analytics — Системная аналитика
```

---

### 4. Plugin Management

**API Endpoints:**
- `GET /api/plugins` — Список плагинов
- `GET /api/plugins/:id` — Информация о плагине
- `POST /api/plugins/:id/enable` — Включить
- `POST /api/plugins/:id/disable` — Отключить
- `PUT /api/plugins/:id/config` — Конфигурация

**Необходимые компоненты:**
- `PluginList.vue` — Список плагинов
- `PluginCard.vue` — Карточка плагина
- `PluginDetails.vue` — Детали плагина
- `PluginConfigForm.vue` — Настройка плагина
- `PluginHookList.vue` — Хуки плагина

**Страницы:**
```
/admin/plugins — Управление плагинами
/project/:id/plugins — Плагины проекта
```

---

### 5. Advanced Task Management

**API Endpoints:**
- `GET /api/tasks/:id/output` — Вывод задачи
- `POST /api/tasks/:id/stop` — Остановка
- `POST /api/tasks/:id/confirm` — Подтверждение
- `POST /api/tasks/:id/reject` — Отклонение

**Необходимые компоненты:**
- `TaskOutputViewer.vue` — Просмотр вывода
- `TaskStopDialog.vue` — Диалог остановки
- `TaskApprovalDialog.vue` — Диалог подтверждения
- `TaskTimeline.vue` — Временная шкала

---

## 📋 План реализации

### Этап 1: API Client Enhancement (3 дня)

**Файлы для создания/обновления:**

```javascript
// web/src/lib/api-client.js
import axios from 'axios';

class SemaphoreAPI {
  constructor() {
    this.client = axios.create({
      baseURL: '/api',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Interceptor для токенов
    this.client.interceptors.request.use(config => {
      const token = localStorage.getItem('semaphore_token');
      if (token) {
        config.headers.Authorization = `Bearer ${token}`;
      }
      return config;
    });

    // Interceptor для ошибок
    this.client.interceptors.response.use(
      response => response,
      error => {
        if (error.response?.status === 401) {
          localStorage.removeItem('semaphore_token');
          window.location.href = '/auth/login';
        }
        return Promise.reject(error);
      }
    );
  }

  // Audit Log API
  async getAuditLogs(params = {}) {
    return (await this.client.get('/audit-log', { params })).data;
  }

  async getAuditLog(id) {
    return (await this.client.get(`/audit-log/${id}`)).data;
  }

  async getProjectAuditLogs(projectId, params = {}) {
    return (await this.client.get(`/project/${projectId}/audit-log`, { params })).data;
  }

  async clearAuditLog() {
    return (await this.client.delete('/audit-log/clear')).data;
  }

  // Webhook API
  async getWebhooks(projectId) {
    return (await this.client.get(`/project/${projectId}/webhooks`)).data;
  }

  async createWebhook(projectId, webhook) {
    return (await this.client.post(`/project/${projectId}/webhooks`, webhook)).data;
  }

  async updateWebhook(projectId, id, webhook) {
    return (await this.client.put(`/project/${projectId}/webhooks/${id}`, webhook)).data;
  }

  async deleteWebhook(projectId, id) {
    return (await this.client.delete(`/project/${projectId}/webhooks/${id}`)).data;
  }

  async testWebhook(projectId, id) {
    return (await this.client.post(`/project/${projectId}/webhooks/${id}/test`)).data;
  }

  // Analytics API
  async getSystemAnalytics() {
    return (await this.client.get('/analytics/system')).data;
  }

  async getProjectAnalytics(projectId, params = {}) {
    return (await this.client.get(`/analytics/project/${projectId}`, { params })).data;
  }

  async getTaskStats(projectId, params = {}) {
    return (await this.client.get(`/analytics/project/${projectId}/tasks`, { params })).data;
  }

  async getUserActivity(projectId, params = {}) {
    return (await this.client.get(`/analytics/project/${projectId}/users`, { params })).data;
  }

  async getPerformanceMetrics(projectId, params = {}) {
    return (await this.client.get(`/analytics/project/${projectId}/performance`, { params })).data;
  }

  async getChartData(projectId, params = {}) {
    return (await this.client.get(`/analytics/project/${projectId}/chart`, { params })).data;
  }

  async getSlowTasks(projectId, params = {}) {
    return (await this.client.get(`/analytics/project/${projectId}/slow-tasks`, { params })).data;
  }

  async getRunnerMetrics() {
    return (await this.client.get('/analytics/runners')).data;
  }

  async getHealthStatus() {
    return (await this.client.get('/analytics/health')).data;
  }

  // Plugin API
  async getPlugins() {
    return (await this.client.get('/plugins')).data;
  }

  async getPlugin(id) {
    return (await this.client.get(`/plugins/${id}`)).data;
  }

  async enablePlugin(id) {
    return (await this.client.post(`/plugins/${id}/enable`)).data;
  }

  async disablePlugin(id) {
    return (await this.client.post(`/plugins/${id}/disable`)).data;
  }

  async updatePluginConfig(id, config) {
    return (await this.client.put(`/plugins/${id}/config`, config)).data;
  }
}

export default new SemaphoreAPI();
```

---

### Этап 2: Audit Log Interface (4 дня)

**Файлы для создания:**

```vue
<!-- web/src/views/project/AuditLog.vue -->
<template>
  <v-container fluid>
    <v-row>
      <v-col>
        <h1 class="text-h4 mb-4">Audit Log</h1>
      </v-col>
    </v-row>

    <v-row>
      <v-col>
        <AuditLogFilter
          v-model="filters"
          @update="$refs.table.refresh()"
        />
      </v-col>
    </v-row>

    <v-row>
      <v-col>
        <v-data-table
          ref="table"
          :headers="headers"
          :items="items"
          :loading="loading"
          :server-items-length="total"
          @update:options="loadItems"
        >
          <template v-slot:item.action="{ item }">
            <v-chip small :color="getActionColor(item.action)">
              {{ item.action }}
            </v-chip>
          </template>

          <template v-slot:item.level="{ item }">
            <v-chip small :color="getLevelColor(item.level)">
              {{ item.level }}
            </v-chip>
          </template>

          <template v-slot:item.created="{ item }">
            {{ formatDate(item.created) }}
          </template>

          <template v-slot:item.actions="{ item }">
            <v-btn
              small
              text
              @click="showDetails(item)"
            >
              Детали
            </v-btn>
          </template>
        </v-data-table>
      </v-col>
    </v-row>

    <AuditLogDetailsDialog
      v-model="detailsDialog"
      :item="selectedItem"
    />
  </v-container>
</template>

<script>
import api from '@/lib/api-client';
import AuditLogFilter from '@/components/AuditLogFilter.vue';
import AuditLogDetailsDialog from '@/components/AuditLogDetailsDialog.vue';

export default {
  components: {
    AuditLogFilter,
    AuditLogDetailsDialog,
  },

  data: () => ({
    filters: {},
    items: [],
    loading: false,
    total: 0,
    detailsDialog: false,
    selectedItem: null,
    headers: [
      { text: 'ID', value: 'id' },
      { text: 'Действие', value: 'action' },
      { text: 'Объект', value: 'object_name' },
      { text: 'Пользователь', value: 'username' },
      { text: 'Уровень', value: 'level' },
      { text: 'Дата', value: 'created' },
      { text: '', value: 'actions', sortable: false },
    ],
  }),

  methods: {
    async loadItems(options) {
      this.loading = true;
      try {
        const params = {
          ...this.filters,
          limit: options.itemsPerPage,
          offset: (options.page - 1) * options.itemsPerPage,
          sort: options.sortBy?.[0],
          order: options.sortDesc?.[0] ? 'desc' : 'asc',
        };

        const response = await api.getProjectAuditLogs(
          this.projectId,
          params
        );

        this.items = response.records;
        this.total = response.total;
      } finally {
        this.loading = false;
      }
    },

    showDetails(item) {
      this.selectedItem = item;
      this.detailsDialog = true;
    },

    getActionColor(action) {
      // Логика выбора цвета
    },

    getLevelColor(level) {
      const colors = {
        info: 'blue',
        warning: 'orange',
        error: 'red',
        critical: 'purple',
      };
      return colors[level] || 'grey';
    },

    formatDate(date) {
      return new Date(date).toLocaleString();
    },
  },
};
</script>
```

---

### Этап 3: Webhook Management (3 дня)

**Файлы для создания:**

```vue
<!-- web/src/views/project/Webhooks.vue -->
<template>
  <v-container fluid>
    <v-row>
      <v-col>
        <div class="d-flex justify-space-between">
          <h1 class="text-h4 mb-4">Webhooks</h1>
          <v-btn color="primary" @click="showCreateDialog">
            <v-icon left>mdi-plus</v-icon>
            Создать Webhook
          </v-btn>
        </div>
      </v-col>
    </v-row>

    <v-row>
      <v-col
        v-for="webhook in webhooks"
        :key="webhook.id"
        cols="12"
        md="6"
        lg="4"
      >
        <WebhookCard
          :webhook="webhook"
          @edit="showEditDialog(webhook)"
          @delete="confirmDelete(webhook)"
          @test="showTestDialog(webhook)"
        />
      </v-col>
    </v-row>

    <WebhookFormDialog
      v-model="formDialog"
      :webhook="selectedWebhook"
      @save="saveWebhook"
    />

    <WebhookTestDialog
      v-model="testDialog"
      :webhook="selectedWebhook"
      @test="testWebhook"
    />

    <YesNoDialog
      v-model="deleteDialog"
      title="Удалить Webhook?"
      @yes="deleteWebhook"
    />
  </v-container>
</template>

<script>
import api from '@/lib/api-client';
import WebhookCard from '@/components/WebhookCard.vue';
import WebhookFormDialog from '@/components/WebhookFormDialog.vue';
import WebhookTestDialog from '@/components/WebhookTestDialog.vue';
import YesNoDialog from '@/components/YesNoDialog.vue';

export default {
  components: {
    WebhookCard,
    WebhookFormDialog,
    WebhookTestDialog,
    YesNoDialog,
  },

  data: () => ({
    webhooks: [],
    loading: false,
    formDialog: false,
    testDialog: false,
    deleteDialog: false,
    selectedWebhook: null,
  }),

  async mounted() {
    await this.loadWebhooks();
  },

  methods: {
    async loadWebhooks() {
      this.loading = true;
      try {
        this.webhooks = await api.getWebhooks(this.projectId);
      } finally {
        this.loading = false;
      }
    },

    showCreateDialog() {
      this.selectedWebhook = null;
      this.formDialog = true;
    },

    showEditDialog(webhook) {
      this.selectedWebhook = webhook;
      this.formDialog = true;
    },

    async saveWebhook(webhook) {
      if (webhook.id) {
        await api.updateWebhook(this.projectId, webhook.id, webhook);
      } else {
        await api.createWebhook(this.projectId, webhook);
      }
      await this.loadWebhooks();
    },

    confirmDelete(webhook) {
      this.selectedWebhook = webhook;
      this.deleteDialog = true;
    },

    async deleteWebhook() {
      await api.deleteWebhook(this.projectId, this.selectedWebhook.id);
      await this.loadWebhooks();
    },

    showTestDialog(webhook) {
      this.selectedWebhook = webhook;
      this.testDialog = true;
    },

    async testWebhook() {
      await api.testWebhook(this.projectId, this.selectedWebhook.id);
    },
  },
};
</script>
```

---

### Этап 4: Analytics Dashboard (5 дней)

**Файлы для создания:**

```vue
<!-- web/src/views/project/Analytics.vue -->
<template>
  <v-container fluid>
    <v-row>
      <v-col>
        <h1 class="text-h4 mb-4">Аналитика</h1>
      </v-col>
    </v-row>

    <!-- Project Stats Cards -->
    <v-row>
      <v-col cols="12" md="3">
        <ProjectStatsCard
          title="Всего задач"
          :value="stats.total_tasks"
          icon="mdi-check-circle"
          color="blue"
        />
      </v-col>
      <v-col cols="12" md="3">
        <ProjectStatsCard
          title="Успешных"
          :value="stats.successful_tasks"
          icon="mdi-check"
          color="green"
        />
      </v-col>
      <v-col cols="12" md="3">
        <ProjectStatsCard
          title="Проваленных"
          :value="stats.failed_tasks"
          icon="mdi-close"
          color="red"
        />
      </v-col>
      <v-col cols="12" md="3">
        <ProjectStatsCard
          title="Процент успеха"
          :value="`${stats.success_rate}%`"
          icon="mdi-percent"
          color="teal"
        />
      </v-col>
    </v-row>

    <!-- Task Metrics Chart -->
    <v-row>
      <v-col cols="12" md="8">
        <v-card>
          <v-card-title>Задачи по времени</v-card-title>
          <v-card-text>
            <TaskMetricsChart :data="taskChartData" />
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" md="4">
        <v-card>
          <v-card-title>Статусы задач</v-card-title>
          <v-card-text>
            <TaskStatusPieChart :data="taskStatusData" />
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- User Activity -->
    <v-row>
      <v-col cols="12">
        <v-card>
          <v-card-title>Активность пользователей</v-card-title>
          <v-card-text>
            <UserActivityChart :data="userActivityData" />
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Performance Metrics -->
    <v-row>
      <v-col cols="12">
        <v-card>
          <v-card-title>Производительность</v-card-title>
          <v-card-text>
            <PerformanceMetrics :metrics="performanceMetrics" />
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Slow Tasks -->
    <v-row>
      <v-col cols="12">
        <v-card>
          <v-card-title>Медленные задачи</v-card-title>
          <v-card-text>
            <SlowTasksTable :tasks="slowTasks" />
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script>
import api from '@/lib/api-client';
import ProjectStatsCard from '@/components/ProjectStatsCard.vue';
import TaskMetricsChart from '@/components/TaskMetricsChart.vue';
import TaskStatusPieChart from '@/components/TaskStatusPieChart.vue';
import UserActivityChart from '@/components/UserActivityChart.vue';
import PerformanceMetrics from '@/components/PerformanceMetrics.vue';
import SlowTasksTable from '@/components/SlowTasksTable.vue';

export default {
  components: {
    ProjectStatsCard,
    TaskMetricsChart,
    TaskStatusPieChart,
    UserActivityChart,
    PerformanceMetrics,
    SlowTasksTable,
  },

  data: () => ({
    stats: {},
    taskChartData: [],
    taskStatusData: [],
    userActivityData: [],
    performanceMetrics: {},
    slowTasks: [],
  }),

  async mounted() {
    await this.loadAnalytics();
  },

  methods: {
    async loadAnalytics() {
      const [
        analytics,
        taskStats,
        userActivity,
        performance,
        chartData,
        slowTasks,
      ] = await Promise.all([
        api.getProjectAnalytics(this.projectId),
        api.getTaskStats(this.projectId),
        api.getUserActivity(this.projectId),
        api.getPerformanceMetrics(this.projectId),
        api.getChartData(this.projectId, { metric: 'tasks', period: 'day' }),
        api.getSlowTasks(this.projectId, { limit: 10 }),
      ]);

      this.stats = analytics.stats;
      this.taskChartData = this.processTaskChartData(chartData);
      this.taskStatusData = this.processTaskStatusData(taskStats);
      this.userActivityData = userActivity;
      this.performanceMetrics = performance;
      this.slowTasks = slowTasks;
    },
  },
};
</script>
```

---

### Этап 5: Plugin Management (3 дня)

**Файлы для создания:**

```vue
<!-- web/src/views/admin/Plugins.vue -->
<template>
  <v-container fluid>
    <v-row>
      <v-col>
        <h1 class="text-h4 mb-4">Плагины</h1>
      </v-col>
    </v-row>

    <v-row>
      <v-col
        v-for="plugin in plugins"
        :key="plugin.id"
        cols="12"
        md="6"
        lg="4"
      >
        <PluginCard
          :plugin="plugin"
          @enable="enablePlugin"
          @disable="disablePlugin"
          @configure="showConfigDialog"
        />
      </v-col>
    </v-row>

    <PluginConfigDialog
      v-model="configDialog"
      :plugin="selectedPlugin"
      @save="savePluginConfig"
    />
  </v-container>
</template>

<script>
import api from '@/lib/api-client';
import PluginCard from '@/components/PluginCard.vue';
import PluginConfigDialog from '@/components/PluginConfigDialog.vue';

export default {
  components: {
    PluginCard,
    PluginConfigDialog,
  },

  data: () => ({
    plugins: [],
    loading: false,
    configDialog: false,
    selectedPlugin: null,
  }),

  async mounted() {
    await this.loadPlugins();
  },

  methods: {
    async loadPlugins() {
      this.loading = true;
      try {
        this.plugins = await api.getPlugins();
      } finally {
        this.loading = false;
      }
    },

    async enablePlugin(plugin) {
      await api.enablePlugin(plugin.id);
      await this.loadPlugins();
    },

    async disablePlugin(plugin) {
      await api.disablePlugin(plugin.id);
      await this.loadPlugins();
    },

    showConfigDialog(plugin) {
      this.selectedPlugin = plugin;
      this.configDialog = true;
    },

    async savePluginConfig(config) {
      await api.updatePluginConfig(
        this.selectedPlugin.id,
        config
      );
      await this.loadPlugins();
    },
  },
};
</script>
```

---

## 📅 Timeline реализации

| Этап | Задачи | Длительность | Приоритет |
|------|--------|--------------|-----------|
| **1** | API Client Enhancement | 3 дня | 🔴 Высокий |
| **2** | Audit Log Interface | 4 дня | 🔴 Высокий |
| **3** | Webhook Management | 3 дня | 🟠 Средний |
| **4** | Analytics Dashboard | 5 дней | 🔴 Высокий |
| **5** | Plugin Management | 3 дня | 🟠 Средний |
| **6** | Тестирование и отладка | 3 дня | 🟠 Средний |
| **7** | Документация | 2 дня | 🟢 Низкий |

**Общая длительность:** 23 рабочих дня (~5 недель)

---

## 🧪 Тестирование

### Unit Tests

```javascript
// web/tests/unit/lib/api-client.spec.js
import { expect } from 'chai';
import api from '@/lib/api-client';

describe('API Client', () => {
  it('should fetch audit logs', async () => {
    const logs = await api.getAuditLogs({ limit: 10 });
    expect(logs).to.be.an('array');
  });

  it('should fetch analytics', async () => {
    const analytics = await api.getProjectAnalytics(1);
    expect(analytics).to.have.property('stats');
  });
});
```

### E2E Tests

```javascript
// web/tests/e2e/specs/audit-log.spec.js
describe('Audit Log', () => {
  it('should display audit log page', () => {
    cy.visit('/project/1/audit-log');
    cy.contains('Audit Log').should('be.visible');
  });

  it('should filter audit logs', () => {
    cy.visit('/project/1/audit-log');
    cy.get('[data-cy="filter-action"]').select('login');
    cy.get('[data-cy="audit-table"]').should('contain', 'login');
  });
});
```

---

## 📚 Документация

### Компоненты

| Компонент | Назначение | Статус |
|-----------|------------|--------|
| `AuditLogFilter.vue` | Фильтры audit log | 📅 Запланировано |
| `AuditLogDetailsDialog.vue` | Детали записи | 📅 Запланировано |
| `WebhookCard.vue` | Карточка webhook | 📅 Запланировано |
| `WebhookFormDialog.vue` | Форма webhook | 📅 Запланировано |
| `WebhookTestDialog.vue` | Тест webhook | 📅 Запланировано |
| `ProjectStatsCard.vue` | Карточка статистики | 📅 Запланировано |
| `TaskMetricsChart.vue` | График задач | 📅 Запланировано |
| `UserActivityChart.vue` | Активность | 📅 Запланировано |
| `PluginCard.vue` | Карточка плагина | 📅 Запланировано |
| `PluginConfigDialog.vue` | Настройка плагина | 📅 Запланировано |

---

*Последнее обновление: 9 марта 2026 г.*
