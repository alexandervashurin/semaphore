<template xmlns:v-slot="http://www.w3.org/1999/XSL/Transform">
  <div v-if="items != null">

    <EditDialog
      v-model="editDialog"
      :save-button-text="itemId === 'new' ? $t('create') : $t('save')"
      icon="mdi-file-document-outline"
      icon-color="primary"
      :title="`${itemId === 'new' ? $t('nnew') : $t('edit')} Playbook`"
      :max-width="600"
      @save="loadItems"
    >
      <template v-slot:form="{ onSave, onError, needSave, needReset }">
        <PlaybookForm
          :project-id="projectId"
          :item-id="itemId"
          @save="onSave"
          @error="onError"
          :need-save="needSave"
          :need-reset="needReset"
        />
      </template>
    </EditDialog>

    <ObjectRefsDialog
      object-title="playbook"
      :object-refs="itemRefs"
      :project-id="projectId"
      v-model="itemRefsDialog"
    />

    <YesNoDialog
      :title="$t('deletePlaybook')"
      :text="$t('askDeletePlaybook')"
      v-model="deleteItemDialog"
      @yes="deleteItem(itemId)"
    />

    <v-toolbar flat>
      <v-app-bar-nav-icon @click="showDrawer()"></v-app-bar-nav-icon>
      <v-toolbar-title>{{ $t('playbooks') }}</v-toolbar-title>
      <v-spacer></v-spacer>

      <v-btn
        class="pr-2"
        color="primary"
        v-if="can(USER_PERMISSIONS.manageProjectResources)"
        @click="openNewDialog()"
      >
        {{ $t('newPlaybook') }}
        <v-icon>mdi-plus</v-icon>
      </v-btn>
    </v-toolbar>

    <v-progress-linear
      v-if="items == null"
      indeterminate
    ></v-progress-linear>

    <v-data-table
      v-else-if="items.length > 0"
      :headers="headers"
      :items="items"
      :items-per-page="25"
      hide-default-footer
      class="elevation-0"
    >
      <template v-slot:item.name="{ item }">
        <div class="d-flex align-center">
          <v-icon class="mr-2" color="primary">mdi-file-document-outline</v-icon>
          <span>{{ item.name }}</span>
        </div>
      </template>

      <template v-slot:item.playbook_type="{ item }">
        <v-chip small :color="getPlaybookTypeColor(item.playbook_type)">
          {{ item.playbook_type }}
        </v-chip>
      </template>

      <template v-slot:item.actions="{ item }">
        <v-menu
          bottom
          left
        >
          <template v-slot:activator="{ on, attrs }">
            <v-btn
              icon
              v-bind="attrs"
              v-on="on"
            >
              <v-icon>mdi-dots-vertical</v-icon>
            </v-btn>
          </template>
          <v-list>
            <v-list-item
              @click="editItemId(item.id)"
              v-if="can(USER_PERMISSIONS.manageProjectResources)"
            >
              <v-list-item-icon>
                <v-icon>mdi-pencil</v-icon>
              </v-list-item-icon>
              <v-list-item-title>{{ $t('edit') }}</v-list-item-title>
            </v-list-item>

            <v-list-item
              @click="runPlaybook(item.id)"
              v-if="can(USER_PERMISSIONS.runTemplate)"
            >
              <v-list-item-icon>
                <v-icon color="green">mdi-play</v-icon>
              </v-list-item-icon>
              <v-list-item-title>{{ $t('run') }}</v-list-item-title>
            </v-list-item>

            <v-list-item
              @click="syncPlaybook(item.id)"
              v-if="item.repository_id && can(USER_PERMISSIONS.manageProjectResources)"
            >
              <v-list-item-icon>
                <v-icon color="blue">mdi-sync</v-icon>
              </v-list-item-icon>
              <v-list-item-title>Sync from Git</v-list-item-title>
            </v-list-item>

            <v-list-item
              @click="showRefs(item)"
            >
              <v-list-item-icon>
                <v-icon>mdi-link-variant</v-icon>
              </v-list-item-icon>
              <v-list-item-title>{{ $t('references') }}</v-list-item-title>
            </v-list-item>

            <v-list-item
              @click="deleteItemId(item.id)"
              v-if="can(USER_PERMISSIONS.manageProjectResources)"
            >
              <v-list-item-icon>
                <v-icon color="error">mdi-delete</v-icon>
              </v-list-item-icon>
              <v-list-item-title class="error--text">{{ $t('delete') }}</v-list-item-title>
            </v-list-item>
          </v-list>
        </v-menu>
      </template>
    </v-data-table>

    <div
      v-else
      class="text-center pa-8"
    >
      <v-icon size="80" color="grey lighten-1">mdi-file-document-outline</v-icon>
      <h3 class="pt-4">{{ $t('noPlaybooks') }}</h3>
      <p class="grey--text">{{ $t('noPlaybooksText') }}</p>
      <v-btn
        color="primary"
        depressed
        class="mt-4"
        v-if="can(USER_PERMISSIONS.manageProjectResources)"
        @click="openNewDialog()"
      >
        {{ $t('newPlaybook') }}
      </v-btn>
    </div>
  </div>
</template>

<script>
import { USER_PERMISSIONS, APP_INVENTORY } from '@/lib/constants';
import PermissionsCheck from '@/components/PermissionsCheck';
import YesNoDialog from '@/components/YesNoDialog.vue';
import EditDialog from '@/components/EditDialog.vue';
import ObjectRefsDialog from '@/components/ObjectRefsDialog.vue';
import ItemListPageBase from '@/components/ItemListPageBase';
import PlaybookForm from '@/components/PlaybookForm.vue';

export default {
  components: {
    YesNoDialog,
    EditDialog,
    ObjectRefsDialog,
    PlaybookForm,
  },
  mixins: [ItemListPageBase, PermissionsCheck],
  data() {
    return {
      USER_PERMISSIONS,
      itemApp: APP_INVENTORY,
      itemId: null,
      items: null,
      deleteItemDialog: false,
      itemRefsDialog: false,
      itemRefs: {},
      editDialog: false,
      attachInventoryDialog: false,
    };
  },
  computed: {
    headers() {
      return [
        {
          text: this.$t('name'),
          value: 'name',
          sortable: true,
        },
        {
          text: this.$t('type'),
          value: 'playbook_type',
          sortable: true,
        },
        {
          text: this.$t('description'),
          value: 'description',
          sortable: false,
        },
        {
          text: '',
          value: 'actions',
          sortable: false,
        },
      ];
    },
  },
  methods: {
    getPlaybookTypeColor(type) {
      const colors = {
        ansible: 'orange',
        terraform: 'green',
        shell: 'blue',
      };
      return colors[type] || 'grey';
    },
    runPlaybook(id) {
      this.$router.push(`/project/${this.projectId}/playbooks/${id}/run`);
    },
    syncPlaybook(id) {
      this.$axios.post(`/api/project/${this.projectId}/playbooks/${id}/sync`).then(() => {
        this.$eventBus.$emit('notification', {
          type: 'success',
          message: 'Playbook synced successfully',
        });
        this.loadItems();
      }).catch((err) => {
        this.$eventBus.$emit('notification', {
          type: 'error',
          message: err.response?.data?.error || 'Failed to sync playbook',
        });
      });
    },
    loadItems() {
      this.$axios.get(`/api/project/${this.projectId}/playbooks`).then((resp) => {
        this.items = resp.data;
      }).catch((err) => {
        this.$eventBus.$emit('notification', {
          type: 'error',
          message: err.response?.data?.error || 'Failed to load playbooks',
        });
      });
    },
    deleteItem(itemId) {
      this.$axios.delete(`/api/project/${this.projectId}/playbooks/${itemId}`).then(() => {
        this.loadItems();
        this.$eventBus.$emit('notification', {
          type: 'success',
          message: 'Playbook deleted',
        });
      }).catch((err) => {
        this.$eventBus.$emit('notification', {
          type: 'error',
          message: err.response?.data?.error || 'Failed to delete playbook',
        });
      });
    },
    showRefs(item) {
      this.$axios.get(`/api/project/${this.projectId}/playbooks/${item.id}/refs`).then((resp) => {
        this.itemRefs = resp.data;
        this.itemRefsDialog = true;
      });
    },
  },
};
</script>
