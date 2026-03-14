<template>
  <v-form
    ref="form"
    v-model="valid"
    lazy-validation
    @submit.prevent
  >
    <v-alert
      :value="!!error"
      color="error"
      class="mb-4"
    >{{ error }}</v-alert>

    <v-text-field
      v-model="item.name"
      :label="$t('name')"
      :rules="[v => !!v || 'Name is required']"
      required
      outlined
      dense
    />

    <v-select
      v-model="item.playbook_type"
      :items="playbookTypes"
      :label="$t('type')"
      :rules="[v => !!v || 'Type is required']"
      required
      outlined
      dense
    />

    <v-select
      v-model="item.repository_id"
      :items="repositories"
      item-text="name"
      item-value="id"
      label="Repository (optional)"
      outlined
      dense
      clearable
    />

    <v-textarea
      v-model="item.description"
      :label="$t('description')"
      outlined
      dense
      rows="2"
    />

    <v-divider class="my-4"></v-divider>

    <div class="d-flex align-center mb-2">
      <v-icon class="mr-2">mdi-code-tags</v-icon>
      <span class="text-subtitle-2">Playbook Content (YAML)</span>
    </div>

    <v-alert
      type="info"
      density="compact"
      class="mb-2"
    >
      <v-icon start>mdi-information</v-icon>
      Paste your Ansible/Terraform/Shell playbook content here
    </v-alert>

    <codemirror
      v-model="item.content"
      :options="cmOptions"
      :height="400"
      class="border rounded"
    />

    <v-checkbox
      v-model="item.validate_content"
      label="Validate YAML syntax"
      class="mt-2"
    />
  </v-form>
</template>

<script>
import { codemirror } from 'vue-codemirror';
import 'codemirror/lib/codemirror.css';
import 'codemirror/mode/yaml/yaml.js';
import 'codemirror/theme/material.css';

export default {
  components: {
    codemirror,
  },
  props: {
    projectId: {
      type: Number,
      required: true,
    },
    itemId: {
      type: [Number, String],
      required: true,
    },
    needSave: Boolean,
    needReset: Boolean,
  },
  data() {
    return {
      valid: true,
      item: {
        name: '',
        playbook_type: 'ansible',
        content: '',
        description: '',
        repository_id: null,
        validate_content: true,
      },
      repositories: [],
      playbookTypes: [
        { text: 'Ansible', value: 'ansible' },
        { text: 'Terraform', value: 'terraform' },
        { text: 'Shell', value: 'shell' },
      ],
      cmOptions: {
        tabSize: 2,
        mode: 'yaml',
        theme: 'material',
        lineNumbers: true,
        line: true,
        lineWrapping: true,
      },
      error: null,
    };
  },
  watch: {
    needSave(val) {
      if (val) {
        this.save();
      }
    },
    needReset(val) {
      if (val) {
        this.reset();
      }
    },
  },
  mounted() {
    this.loadRepositories();
    if (this.itemId !== 'new') {
      this.loadItem();
    }
  },
  methods: {
    loadRepositories() {
      this.$axios.get(`/api/project/${this.projectId}/repositories`).then((resp) => {
        this.repositories = resp.data;
      });
    },
    loadItem() {
      this.$axios.get(`/api/project/${this.projectId}/playbooks/${this.itemId}`).then((resp) => {
        this.item = {
          ...resp.data,
          validate_content: true,
        };
      }).catch((err) => {
        this.error = err.response?.data?.error || 'Failed to load playbook';
      });
    },
    save() {
      if (!this.$refs.form.validate()) {
        this.$emit('error', 'Form is invalid');
        return;
      }

      // Validate YAML if enabled
      if (this.item.validate_content && this.item.content) {
        try {
          // Simple YAML validation
          if (!this.item.content.includes('---') && !this.item.content.includes(':')) {
            throw new Error('Invalid YAML content');
          }
        } catch (e) {
          this.error = 'Invalid YAML content: ' + e.message;
          this.$emit('error', this.error);
          return;
        }
      }

      const payload = {
        name: this.item.name,
        playbook_type: this.item.playbook_type,
        content: this.item.content,
        description: this.item.description,
        repository_id: this.item.repository_id,
      };

      const url = `/api/project/${this.projectId}/playbooks`;
      const method = this.itemId === 'new' ? 'post' : 'put';
      const fullUrl = this.itemId === 'new' ? url : `${url}/${this.itemId}`;

      this.$axios[method](fullUrl, payload).then((resp) => {
        this.$emit('save', resp.data);
      }).catch((err) => {
        this.error = err.response?.data?.error || 'Failed to save playbook';
        this.$emit('error', this.error);
      });
    },
    reset() {
      this.item = {
        name: '',
        playbook_type: 'ansible',
        content: '',
        description: '',
        repository_id: null,
        validate_content: true,
      };
      this.error = null;
      this.$refs.form.resetValidation();
    },
  },
};
</script>

<style scoped>
.CodeMirror {
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  font-size: 13px;
}
</style>
