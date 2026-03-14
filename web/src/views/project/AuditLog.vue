<template xmlns:v-slot="http://www.w3.org/1999/XSL/Transform">
  <div>
    <v-toolbar flat>
      <v-app-bar-nav-icon @click="showDrawer()"></v-app-bar-nav-icon>
      <v-toolbar-title>{{ $t('audit_log') }}</v-toolbar-title>
      <v-spacer></v-spacer>
      <v-btn color="primary" @click="loadAuditLogs" :loading="loading">
        {{ $t('refresh') }}
      </v-btn>
    </v-toolbar>

    <v-card class="mt-4">
      <v-card-title>
        <v-row>
          <v-col cols="12" sm="4">
            <v-text-field
              v-model="search"
              label="Search"
              prepend-inner-icon="mdi-magnify"
              outlined
              dense
              hide-details
            ></v-text-field>
          </v-col>
          <v-col cols="12" sm="3">
            <v-select
              v-model="filter.action"
              :items="actionItems"
              label="Action"
              outlined
              dense
              hide-details
              clearable
            ></v-select>
          </v-col>
          <v-col cols="12" sm="3">
            <v-select
              v-model="filter.level"
              :items="levelItems"
              label="Level"
              outlined
              dense
              hide-details
              clearable
            ></v-select>
          </v-col>
          <v-col cols="12" sm="2">
            <v-select
              v-model="filter.limit"
              :items="[25, 50, 100, 200]"
              label="Limit"
              outlined
              dense
              hide-details
            ></v-select>
          </v-col>
        </v-row>
      </v-card-title>

      <v-data-table
        :headers="headers"
        :items="auditLogs"
        :loading="loading"
        :items-per-page="filter.limit"
        class="elevation-0"
        hide-default-footer
      >
        <template v-slot:item.created="{ item }">
          {{ formatDate(item.created) }}
        </template>

        <template v-slot:item.level="{ item }">
          <v-chip :color="getLevelColor(item.level)" small>
            {{ item.level }}
          </v-chip>
        </template>

        <template v-slot:item.action="{ item }">
          <v-chip :color="getActionColor(item.action)" small>
            {{ formatAction(item.action) }}
          </v-chip>
        </template>

        <template v-slot:item.object_type="{ item }">
          <v-icon small left>{{ getObjectTypeIcon(item.object_type) }}</v-icon>
          {{ formatObjectType(item.object_type) }}
        </template>

        <template v-slot:item.user="{ item }">
          <v-avatar size="24" class="mr-2">
            <v-icon small>mdi-account</v-icon>
          </v-avatar>
          {{ item.username || 'System' }}
        </template>

        <template v-slot:item.description="{ item }">
          <div class="text-truncate" style="max-width: 400px;">
            {{ item.description }}
          </div>
        </template>

        <template v-slot:item.actions="{ item }">
          <v-icon small @click="showDetails(item)">mdi-eye</v-icon>
        </template>
      </v-data-table>

      <v-card-actions v-if="total > filter.limit">
        <v-spacer></v-spacer>
        <v-pagination
          v-model="pagination.page"
          :length="Math.ceil(total / filter.limit)"
          :total-visible="5"
          @input="loadAuditLogs"
        ></v-pagination>
      </v-card-actions>
    </v-card>

    <!-- Details Dialog -->
    <v-dialog v-model="detailsDialog" max-width="600px">
      <v-card>
        <v-card-title>
          <span class="headline">Audit Log Details</span>
        </v-card-title>
        <v-card-text>
          <v-list dense>
            <v-list-item>
              <v-list-item-title class="font-weight-bold">ID</v-list-item-title>
              <v-list-item-subtitle>{{ selectedItem?.id }}</v-list-item-subtitle>
            </v-list-item>
            <v-list-item>
              <v-list-item-title class="font-weight-bold">Timestamp</v-list-item-title>
              <v-list-item-subtitle>{{ selectedItem?.created }}</v-list-item-subtitle>
            </v-list-item>
            <v-list-item>
              <v-list-item-title class="font-weight-bold">User</v-list-item-title>
              <v-list-item-subtitle>{{ selectedItem?.username || 'System' }}</v-list-item-subtitle>
            </v-list-item>
            <v-list-item>
              <v-list-item-title class="font-weight-bold">Action</v-list-item-title>
              <v-list-item-subtitle>{{ selectedItem?.action }}</v-list-item-subtitle>
            </v-list-item>
            <v-list-item>
              <v-list-item-title class="font-weight-bold">Object Type</v-list-item-title>
              <v-list-item-subtitle>{{ selectedItem?.object_type }}</v-list-item-subtitle>
            </v-list-item>
            <v-list-item>
              <v-list-item-title class="font-weight-bold">Object ID</v-list-item-title>
              <v-list-item-subtitle>{{ selectedItem?.object_id || 'N/A' }}</v-list-item-subtitle>
            </v-list-item>
            <v-list-item>
              <v-list-item-title class="font-weight-bold">Level</v-list-item-title>
              <v-list-item-subtitle>
                <v-chip :color="getLevelColor(selectedItem?.level)" small>
                  {{ selectedItem?.level }}
                </v-chip>
              </v-list-item-subtitle>
            </v-list-item>
            <v-list-item>
              <v-list-item-title class="font-weight-bold">IP Address</v-list-item-title>
              <v-list-item-subtitle>{{ selectedItem?.ip_address || 'N/A' }}</v-list-item-subtitle>
            </v-list-item>
            <v-list-item>
              <v-list-item-title class="font-weight-bold">Description</v-list-item-title>
              <v-list-item-subtitle>{{ selectedItem?.description }}</v-list-item-subtitle>
            </v-list-item>
            <v-list-item v-if="selectedItem?.details">
              <v-list-item-title class="font-weight-bold">Details</v-list-item-title>
              <v-list-item-subtitle>
                <pre class="json-pre">{{ JSON.stringify(selectedItem.details, null, 2) }}</pre>
              </v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="primary" text @click="detailsDialog = false">Close</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
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
    userId: Number,
    userRole: String,
  },

  data() {
    return {
      loading: false,
      auditLogs: [],
      total: 0,
      search: '',
      filter: {
        action: null,
        level: null,
        limit: 50,
      },
      pagination: {
        page: 1,
      },
      detailsDialog: false,
      selectedItem: null,
      headers: [
        { text: 'Time', value: 'created', sortable: true },
        { text: 'Level', value: 'level', sortable: true },
        { text: 'Action', value: 'action', sortable: true },
        { text: 'Object', value: 'object_type', sortable: true },
        { text: 'User', value: 'user', sortable: false },
        { text: 'Description', value: 'description', sortable: false },
        { text: 'Actions', value: 'actions', sortable: false, align: 'end' },
      ],
    };
  },

  computed: {
    actionItems() {
      return [
        { text: 'Login', value: 'login' },
        { text: 'Logout', value: 'logout' },
        { text: 'Login Failed', value: 'login_failed' },
        { text: 'Password Changed', value: 'password_changed' },
        { text: 'User Created', value: 'user_created' },
        { text: 'User Updated', value: 'user_updated' },
        { text: 'User Deleted', value: 'user_deleted' },
        { text: 'Project Created', value: 'project_created' },
        { text: 'Project Updated', value: 'project_updated' },
        { text: 'Project Deleted', value: 'project_deleted' },
        { text: 'Task Created', value: 'task_created' },
        { text: 'Task Started', value: 'task_started' },
        { text: 'Task Completed', value: 'task_completed' },
        { text: 'Task Failed', value: 'task_failed' },
        { text: 'Task Deleted', value: 'task_deleted' },
        { text: 'Template Created', value: 'template_created' },
        { text: 'Template Updated', value: 'template_updated' },
        { text: 'Template Deleted', value: 'template_deleted' },
        { text: 'Template Run', value: 'template_run' },
      ];
    },

    levelItems() {
      return [
        { text: 'Info', value: 'info' },
        { text: 'Warning', value: 'warning' },
        { text: 'Error', value: 'error' },
        { text: 'Critical', value: 'critical' },
      ];
    },
  },

  mounted() {
    this.loadAuditLogs();
  },

  methods: {
    async loadAuditLogs() {
      this.loading = true;
      try {
        const params = {
          limit: this.filter.limit,
          offset: (this.pagination.page - 1) * this.filter.limit,
        };

        if (this.search) {
          params.search = this.search;
        }
        if (this.filter.action) {
          params.action = this.filter.action;
        }
        if (this.filter.level) {
          params.level = this.filter.level;
        }

        const response = await axios.get(`/api/audit-log`, { params });
        
        if (response.data && response.data.records) {
          this.auditLogs = response.data.records;
          this.total = response.data.total || response.data.records.length;
        } else {
          this.auditLogs = [];
          this.total = 0;
        }
      } catch (error) {
        console.error('Failed to load audit logs:', error);
        this.$emit('error', error);
      } finally {
        this.loading = false;
      }
    },

    showDetails(item) {
      this.selectedItem = item;
      this.detailsDialog = true;
    },

    formatDate(dateString) {
      if (!dateString) return '';
      const date = new Date(dateString);
      return date.toLocaleString();
    },

    formatAction(action) {
      if (!action) return '';
      return action.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
    },

    formatObjectType(objectType) {
      if (!objectType) return '';
      return objectType.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
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

    getActionColor(action) {
      if (!action) return 'grey';
      if (action.includes('created')) return 'green';
      if (action.includes('updated')) return 'blue';
      if (action.includes('deleted')) return 'red';
      if (action.includes('failed') || action.includes('error')) return 'orange';
      if (action.includes('success') || action.includes('completed')) return 'green';
      return 'grey';
    },

    getObjectTypeIcon(objectType) {
      const icons = {
        user: 'mdi-account',
        project: 'mdi-folder',
        task: 'mdi-play',
        template: 'mdi-file-document',
        inventory: 'mdi-server',
        repository: 'mdi-git',
        environment: 'mdi-settings',
        access_key: 'mdi-key',
        integration: 'mdi-webhook',
        schedule: 'mdi-clock-outline',
      };
      return icons[objectType] || 'mdi-circle-outline';
    },

    showDrawer() {
      this.$emit('drawer');
    },
  },
};
</script>

<style scoped>
.json-pre {
  background: #f5f5f5;
  padding: 10px;
  border-radius: 4px;
  font-size: 12px;
  max-height: 200px;
  overflow: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
}
</style>
