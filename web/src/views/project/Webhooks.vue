<template xmlns:v-slot="http://www.w3.org/1999/XSL/Transform">
  <div>
    <v-toolbar flat>
      <v-app-bar-nav-icon @click="showDrawer()"></v-app-bar-nav-icon>
      <v-toolbar-title>{{ $t('webhooks') }}</v-toolbar-title>
      <v-spacer></v-spacer>
      <v-btn color="primary" @click="showCreateDialog">
        <v-icon left>mdi-plus</v-icon>
        {{ $t('add_webhook') }}
      </v-btn>
    </v-toolbar>

    <v-card class="mt-4">
      <v-card-title>
        <v-row>
          <v-col cols="12" sm="6">
            <v-text-field
              v-model="search"
              label="Search webhooks"
              prepend-inner-icon="mdi-magnify"
              outlined
              dense
              hide-details
              @input="loadWebhooks"
            ></v-text-field>
          </v-col>
          <v-col cols="12" sm="3">
            <v-select
              v-model="filter.type"
              :items="typeItems"
              label="Type"
              outlined
              dense
              hide-details
              clearable
              @change="loadWebhooks"
            ></v-select>
          </v-col>
          <v-col cols="12" sm="3">
            <v-select
              v-model="filter.active"
              :items="activeItems"
              label="Status"
              outlined
              dense
              hide-details
              clearable
              @change="loadWebhooks"
            ></v-select>
          </v-col>
        </v-row>
      </v-card-title>

      <v-data-table
        :headers="headers"
        :items="webhooks"
        :loading="loading"
        class="elevation-0"
        hide-default-footer
      >
        <template v-slot:item.active="{ item }">
          <v-chip :color="item.active ? 'green' : 'grey'" small>
            {{ item.active ? 'Active' : 'Inactive' }}
          </v-chip>
        </template>

        <template v-slot:item.type="{ item }">
          <v-icon small left>{{ getTypeIcon(item.type) }}</v-icon>
          {{ formatType(item.type) }}
        </template>

        <template v-slot:item.events="{ item }">
          <v-chip-group column>
            <v-chip v-for="event in item.events" :key="event" small color="blue" text-color="white">
              {{ event }}
            </v-chip>
          </v-chip-group>
        </template>

        <template v-slot:item.actions="{ item }">
          <v-icon small class="mr-2" @click="testWebhook(item)">mdi-play</v-icon>
          <v-icon small class="mr-2" @click="editWebhook(item)">mdi-pencil</v-icon>
          <v-icon small @click="deleteWebhook(item)">mdi-delete</v-icon>
        </template>
      </v-data-table>
    </v-card>

    <!-- Create/Edit Dialog -->
    <v-dialog v-model="formDialog" max-width="700px" persistent>
      <v-card>
        <v-card-title>
          <span class="headline">{{ editMode ? 'Edit' : 'Create' }} Webhook</span>
        </v-card-title>
        <v-card-text>
          <v-form ref="form" v-model="formValid">
            <v-row>
              <v-col cols="12">
                <v-text-field
                  v-model="form.name"
                  label="Name"
                  :rules="[v => !!v || 'Name is required']"
                  outlined
                  dense
                ></v-text-field>
              </v-col>
              <v-col cols="12" sm="6">
                <v-select
                  v-model="form.type"
                  :items="typeItems"
                  label="Type"
                  :rules="[v => !!v || 'Type is required']"
                  outlined
                  dense
                ></v-select>
              </v-col>
              <v-col cols="12" sm="6">
                <v-switch
                  v-model="form.active"
                  label="Active"
                  dense
                ></v-switch>
              </v-col>
              <v-col cols="12">
                <v-text-field
                  v-model="form.url"
                  label="Webhook URL"
                  :rules="[v => !!v || 'URL is required', v => /^https?:\/\//.test(v) || 'Must start with http:// or https://']"
                  outlined
                  dense
                ></v-text-field>
              </v-col>
              <v-col cols="12">
                <v-text-field
                  v-model="form.secret"
                  label="Secret (for HMAC signature)"
                  type="password"
                  outlined
                  dense
                  hint="Optional secret for signing requests"
                ></v-text-field>
              </v-col>
              <v-col cols="12">
                <v-textarea
                  v-model="form.headersJson"
                  label="Custom Headers (JSON)"
                  outlined
                  dense
                  hint='Example: {"Authorization": "Bearer token"}'
                ></v-textarea>
              </v-col>
              <v-col cols="12">
                <v-combobox
                  v-model="form.events"
                  :items="eventItems"
                  label="Events"
                  multiple
                  chips
                  outlined
                  dense
                  hint="Select events to trigger this webhook"
                ></v-combobox>
              </v-col>
              <v-col cols="12" sm="6">
                <v-text-field
                  v-model.number="form.retry_count"
                  label="Retry Count"
                  type="number"
                  min="0"
                  max="10"
                  outlined
                  dense
                ></v-text-field>
              </v-col>
              <v-col cols="12" sm="6">
                <v-text-field
                  v-model.number="form.timeout_secs"
                  label="Timeout (seconds)"
                  type="number"
                  min="1"
                  max="300"
                  outlined
                  dense
                ></v-text-field>
              </v-col>
            </v-row>
          </v-form>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn text @click="closeForm">Cancel</v-btn>
          <v-btn color="primary" :loading="saving" @click="saveWebhook">Save</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Test Result Dialog -->
    <v-dialog v-model="testDialog" max-width="500px">
      <v-card>
        <v-card-title>
          <span class="headline">Test Result</span>
        </v-card-title>
        <v-card-text>
          <v-alert :type="testResult.success ? 'success' : 'error'" text>
            {{ testResult.message }}
          </v-alert>
          <pre v-if="testResult.error" class="json-pre">{{ testResult.error }}</pre>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="primary" text @click="testDialog = false">Close</v-btn>
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
      saving: false,
      webhooks: [],
      search: '',
      filter: {
        type: null,
        active: null,
      },
      formDialog: false,
      editMode: false,
      formValid: false,
      form: {
        id: null,
        name: '',
        type: 'generic',
        url: '',
        secret: '',
        active: true,
        events: [],
        retry_count: 3,
        timeout_secs: 30,
        headersJson: '',
      },
      testDialog: false,
      testResult: {
        success: false,
        message: '',
        error: null,
      },
      headers: [
        { text: 'Name', value: 'name', sortable: true },
        { text: 'Type', value: 'type', sortable: true },
        { text: 'URL', value: 'url', sortable: false },
        { text: 'Events', value: 'events', sortable: false },
        { text: 'Status', value: 'active', sortable: true },
        { text: 'Actions', value: 'actions', sortable: false, align: 'end' },
      ],
    };
  },

  computed: {
    typeItems() {
      return [
        { text: 'Generic', value: 'generic' },
        { text: 'Slack', value: 'slack' },
        { text: 'Microsoft Teams', value: 'teams' },
        { text: 'Discord', value: 'discord' },
        { text: 'Telegram', value: 'telegram' },
        { text: 'Custom', value: 'custom' },
      ];
    },

    activeItems() {
      return [
        { text: 'Active', value: true },
        { text: 'Inactive', value: false },
      ];
    },

    eventItems() {
      return [
        'task.created',
        'task.started',
        'task.completed',
        'task.failed',
        'project.created',
        'project.updated',
        'project.deleted',
        'user.created',
        'user.updated',
        'user.deleted',
      ];
    },
  },

  mounted() {
    this.loadWebhooks();
  },

  methods: {
    async loadWebhooks() {
      this.loading = true;
      try {
        const params = {};
        if (this.filter.type) params.type = this.filter.type;
        if (this.filter.active !== null) params.active = this.filter.active;

        const response = await axios.get(`/api/projects/${this.projectId}/webhooks`, { params });
        this.webhooks = response.data || [];
      } catch (error) {
        console.error('Failed to load webhooks:', error);
        this.$emit('error', error);
      } finally {
        this.loading = false;
      }
    },

    showCreateDialog() {
      this.editMode = false;
      this.form = {
        id: null,
        name: '',
        type: 'generic',
        url: '',
        secret: '',
        active: true,
        events: [],
        retry_count: 3,
        timeout_secs: 30,
        headersJson: '',
      };
      this.formDialog = true;
    },

    editWebhook(item) {
      this.editMode = true;
      this.form = {
        id: item.id,
        name: item.name,
        type: item.type,
        url: item.url,
        secret: item.secret || '',
        active: item.active,
        events: item.events || [],
        retry_count: item.retry_count || 3,
        timeout_secs: item.timeout_secs || 30,
        headersJson: item.headers ? JSON.stringify(item.headers) : '',
      };
      this.formDialog = true;
    },

    async saveWebhook() {
      if (!this.$refs.form.validate()) return;

      this.saving = true;
      try {
        const payload = {
          project_id: this.projectId,
          name: this.form.name,
          type: this.form.type,
          url: this.form.url,
          secret: this.form.secret || null,
          active: this.form.active,
          events: this.form.events,
          retry_count: this.form.retry_count,
          timeout_secs: this.form.timeout_secs,
          headers: this.form.headersJson ? JSON.parse(this.form.headersJson) : null,
        };

        if (this.editMode) {
          await axios.put(`/api/projects/${this.projectId}/webhooks/${this.form.id}`, payload);
          this.$emit('success', 'Webhook updated');
        } else {
          await axios.post(`/api/projects/${this.projectId}/webhooks`, payload);
          this.$emit('success', 'Webhook created');
        }

        this.formDialog = false;
        this.loadWebhooks();
      } catch (error) {
        console.error('Failed to save webhook:', error);
        this.$emit('error', error);
      } finally {
        this.saving = false;
      }
    },

    closeForm() {
      this.formDialog = false;
    },

    async testWebhook(item) {
      try {
        const response = await axios.post(`/api/projects/${this.projectId}/webhooks/${item.id}/test`, {});
        this.testResult = {
          success: response.data.success,
          message: response.data.message,
          error: response.data.error,
        };
      } catch (error) {
        this.testResult = {
          success: false,
          message: 'Test failed',
          error: error.response?.data?.error || error.message,
        };
      }
      this.testDialog = true;
    },

    async deleteWebhook(item) {
      if (!confirm(`Delete webhook "${item.name}"?`)) return;

      try {
        await axios.delete(`/api/projects/${this.projectId}/webhooks/${item.id}`);
        this.$emit('success', 'Webhook deleted');
        this.loadWebhooks();
      } catch (error) {
        console.error('Failed to delete webhook:', error);
        this.$emit('error', error);
      }
    },

    formatType(type) {
      if (!type) return '';
      return type.charAt(0).toUpperCase() + type.slice(1);
    },

    getTypeIcon(type) {
      const icons = {
        generic: 'mdi-webhook',
        slack: 'mdi-slack',
        teams: 'mdi-microsoft-teams',
        discord: 'mdi-discord',
        telegram: 'mdi-telegram',
        custom: 'mdi-code-tags',
      };
      return icons[type] || 'mdi-webhook';
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
