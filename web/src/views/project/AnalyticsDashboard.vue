<template xmlns:v-slot="http://www.w3.org/1999/XSL/Transform">
  <div>
    <v-toolbar flat>
      <v-app-bar-nav-icon @click="showDrawer()"></v-app-bar-nav-icon>
      <v-toolbar-title>{{ $t('analytics') }}</v-toolbar-title>
      <v-spacer></v-spacer>
      <v-select
        v-model="dateRange"
        :items="dateRanges"
        label="Period"
        dense
        style="max-width: 200px"
        @change="loadAnalytics"
      ></v-select>
    </v-toolbar>

    <v-row class="mt-4">
      <!-- Summary Cards -->
      <v-col cols="12" sm="6" md="3">
        <v-card color="blue lighten-4">
          <v-card-title class="text-subtitle-1">Total Tasks</v-card-title>
          <v-card-text class="text-h3 font-weight-bold">
            {{ analytics.total_tasks || 0 }}
          </v-card-text>
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-icon>mdi-play</v-icon>
          </v-card-actions>
        </v-card>
      </v-col>

      <v-col cols="12" sm="6" md="3">
        <v-card color="green lighten-4">
          <v-card-title class="text-subtitle-1">Success Rate</v-card-title>
          <v-card-text class="text-h3 font-weight-bold">
            {{ successRate }}%
          </v-card-text>
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-icon>mdi-check-circle</v-icon>
          </v-card-actions>
        </v-card>
      </v-col>

      <v-col cols="12" sm="6" md="3">
        <v-card color="orange lighten-4">
          <v-card-title class="text-subtitle-1">Failed Tasks</v-card-title>
          <v-card-text class="text-h3 font-weight-bold">
            {{ analytics.failed_tasks || 0 }}
          </v-card-text>
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-icon>mdi-alert-circle</v-icon>
          </v-card-actions>
        </v-card>
      </v-col>

      <v-col cols="12" sm="6" md="3">
        <v-card color="purple lighten-4">
          <v-card-title class="text-subtitle-1">Avg Duration</v-card-title>
          <v-card-text class="text-h3 font-weight-bold">
            {{ avgDuration }}
          </v-card-text>
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-icon>mdi-timer</v-icon>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>

    <!-- Charts Row -->
    <v-row>
      <v-col cols="12" md="8">
        <v-card>
          <v-card-title>Tasks Over Time</v-card-title>
          <v-card-text>
            <div ref="tasksChart" style="height: 300px;"></div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="4">
        <v-card>
          <v-card-title>Task Status Distribution</v-card-title>
          <v-card-text>
            <div ref="statusChart" style="height: 300px;"></div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Project Stats -->
    <v-row>
      <v-col cols="12" md="6">
        <v-card>
          <v-card-title>Templates by Type</v-card-title>
          <v-card-text>
            <v-list dense>
              <v-list-item v-for="(count, type) in templatesByType" :key="type">
                <v-list-item-title>{{ formatTemplateType(type) }}</v-list-item-title>
                <v-list-item-action>
                  <v-chip small color="primary" text-color="white">{{ count }}</v-chip>
                </v-list-item-action>
              </v-list-item>
            </v-list>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="6">
        <v-card>
          <v-card-title>Top Users by Activity</v-card-title>
          <v-card-text>
            <v-list dense>
              <v-list-item v-for="user in topUsers" :key="user.id">
                <v-list-item-avatar>
                  <v-icon>mdi-account</v-icon>
                </v-list-item-avatar>
                <v-list-item-content>
                  <v-list-item-title>{{ user.name }}</v-list-item-title>
                  <v-list-item-subtitle>{{ user.tasks_count }} tasks</v-list-item-subtitle>
                </v-list-item-content>
              </v-list-item>
            </v-list>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Recent Activity -->
    <v-card class="mt-4">
      <v-card-title>Recent Activity</v-card-title>
      <v-data-table
        :headers="recentHeaders"
        :items="recentActivity"
        :loading="loading"
        class="elevation-0"
        hide-default-footer
      >
        <template v-slot:item.status="{ item }">
          <v-chip :color="getStatusColor(item.status)" small>
            {{ item.status }}
          </v-chip>
        </template>
        <template v-slot:item.duration="{ item }">
          {{ formatDuration(item.duration) }}
        </template>
      </v-data-table>
    </v-card>
  </div>
</template>

<script>
import axios from 'axios';
import PermissionsCheck from '@/components/PermissionsCheck';

export default {
  mixins: [PermissionsCheck],

  props: {
    projectId: Number,
    projectType: String,
  },

  data() {
    return {
      loading: false,
      dateRange: 'last_week',
      dateRanges: [
        { text: 'Last 7 days', value: 'last_week' },
        { text: 'Last 30 days', value: 'last_month' },
        { text: 'Last 90 days', value: 'last_quarter' },
        { text: 'Last year', value: 'last_year' },
      ],
      analytics: {
        total_tasks: 0,
        success_tasks: 0,
        failed_tasks: 0,
        avg_duration: 0,
        templates_count: 0,
        users_count: 0,
      },
      tasksByDate: [],
      statusDistribution: [],
      templatesByType: {},
      topUsers: [],
      recentActivity: [],
      recentHeaders: [
        { text: 'Task', value: 'task_name' },
        { text: 'Template', value: 'template_name' },
        { text: 'User', value: 'user_name' },
        { text: 'Status', value: 'status' },
        { text: 'Duration', value: 'duration' },
        { text: 'Date', value: 'created' },
      ],
    };
  },

  computed: {
    successRate() {
      if (!this.analytics.total_tasks) return 0;
      return Math.round((this.analytics.success_tasks / this.analytics.total_tasks) * 100);
    },

    avgDuration() {
      const seconds = this.analytics.avg_duration || 0;
      if (seconds < 60) return `${Math.round(seconds)}s`;
      if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
      return `${Math.round(seconds / 3600)}h`;
    },
  },

  mounted() {
    this.loadAnalytics();
  },

  methods: {
    async loadAnalytics() {
      this.loading = true;
      try {
        const [statsResponse, tasksResponse] = await Promise.all([
          axios.get(`/api/projects/${this.projectId}/analytics/stats`, {
            params: { period: this.dateRange }
          }),
          axios.get(`/api/projects/${this.projectId}/analytics/tasks`, {
            params: { period: this.dateRange }
          })
        ]);

        this.analytics = statsResponse.data || {};
        this.tasksByDate = tasksResponse.data?.by_date || [];
        this.statusDistribution = tasksResponse.data?.by_status || [];
        this.templatesByType = statsResponse.data?.templates_by_type || {};
        this.topUsers = statsResponse.data?.top_users || [];
        this.recentActivity = tasksResponse.data?.recent || [];

        this.$nextTick(() => {
          this.renderCharts();
        });
      } catch (error) {
        console.error('Failed to load analytics:', error);
        this.$emit('error', error);
      } finally {
        this.loading = false;
      }
    },

    renderCharts() {
      this.renderTasksChart();
      this.renderStatusChart();
    },

    renderTasksChart() {
      // Упрощённая визуализация - в реальности использовать Chart.js
      const container = this.$refs.tasksChart;
      if (!container) return;

      const data = this.tasksByDate;
      const maxValue = Math.max(...data.map(d => d.count), 1);
      
      let html = '<div style="display: flex; align-items: flex-end; height: 100%; gap: 4px; padding: 10px;">';
      data.forEach(d => {
        const height = (d.count / maxValue) * 100;
        html += `<div style="flex: 1; background: #1976d2; height: ${height}%; border-radius: 4px 4px 0 0;" 
                   title="${d.date}: ${d.count} tasks"></div>`;
      });
      html += '</div>';
      
      container.innerHTML = html;
    },

    renderStatusChart() {
      const container = this.$refs.statusChart;
      if (!container) return;

      const data = this.statusDistribution;
      const total = data.reduce((sum, d) => sum + d.count, 0) || 1;
      
      let html = '<div style="display: flex; flex-direction: column; gap: 8px; padding: 10px;">';
      data.forEach(d => {
        const percent = Math.round((d.count / total) * 100);
        const color = this.getStatusColor(d.status);
        html += `
          <div>
            <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
              <span>${d.status}</span>
              <span>${percent}%</span>
            </div>
            <div style="background: #e0e0e0; height: 8px; border-radius: 4px; overflow: hidden;">
              <div style="background: ${color}; width: ${percent}%; height: 100%;"></div>
            </div>
          </div>
        `;
      });
      html += '</div>';
      
      container.innerHTML = html;
    },

    formatTemplateType(type) {
      if (!type) return 'Unknown';
      return type.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
    },

    formatDuration(seconds) {
      if (!seconds) return 'N/A';
      if (seconds < 60) return `${Math.round(seconds)}s`;
      if (seconds < 3600) return `${Math.round(seconds / 60)}m ${Math.round(seconds % 60)}s`;
      return `${Math.round(seconds / 3600)}h ${Math.round((seconds % 3600) / 60)}m`;
    },

    getStatusColor(status) {
      const colors = {
        success: '#4caf50',
        failed: '#f44336',
        running: '#2196f3',
        waiting: '#ff9800',
      };
      return colors[status?.toLowerCase()] || '#9e9e9e';
    },

    showDrawer() {
      this.$emit('drawer');
    },
  },
};
</script>
