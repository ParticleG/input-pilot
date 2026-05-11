<script setup lang="ts">
import { computed, reactive, ref } from 'vue';
import { useAppStore } from 'stores/app';
import { useQuasar } from 'quasar';
import type { TargetSpec } from 'src/types';

const $q = useQuasar();
const app = useAppStore();

const targets = computed(() => {
  return Object.entries(app.targets).map(([name, t]) => ({
    name,
    process: t.process_name,
    windowClass: t.window_class,
    windowTitle: t.window_title,
    matchMode: t.title_match_mode,
  }));
});

// -- Dialog state -----------------------------------------------------------
const showDialog = ref(false);
const isEditing = ref(false);
const editingOriginalName = ref('');

const matchModeOptions = [
  { label: 'Ignore', value: 'Ignore' as const },
  { label: 'Exact', value: 'Exact' as const },
  { label: 'Contains', value: 'Contains' as const },
];

const form = reactive<TargetSpec>({
  name: '',
  process_name: '',
  window_class: '',
  window_title: '',
  title_match_mode: 'Contains',
  top_level_only: true,
  visible_only: true,
});

function resetForm() {
  form.name = '';
  form.process_name = '';
  form.window_class = '';
  form.window_title = '';
  form.title_match_mode = 'Contains';
  form.top_level_only = true;
  form.visible_only = true;
}

function openCreate() {
  resetForm();
  isEditing.value = false;
  showDialog.value = true;
}

function openEdit(name: string) {
  const t = app.targets[name];
  if (!t) return;
  form.name = t.name;
  form.process_name = t.process_name;
  form.window_class = t.window_class;
  form.window_title = t.window_title;
  form.title_match_mode = t.title_match_mode;
  form.top_level_only = t.top_level_only;
  form.visible_only = t.visible_only;
  editingOriginalName.value = name;
  isEditing.value = true;
  showDialog.value = true;
}

function saveTarget() {
  if (!form.name.trim()) {
    $q.notify({ message: 'Name is required', color: 'negative', position: 'bottom-right' });
    return;
  }
  // Check name collision when creating or renaming
  if (!isEditing.value && app.targets[form.name]) {
    $q.notify({ message: `Target "${form.name}" already exists`, color: 'negative', position: 'bottom-right' });
    return;
  }
  if (isEditing.value && form.name !== editingOriginalName.value && app.targets[form.name]) {
    $q.notify({ message: `Target "${form.name}" already exists`, color: 'negative', position: 'bottom-right' });
    return;
  }

  const spec: TargetSpec = { ...form };
  if (isEditing.value) {
    app.updateTarget(editingOriginalName.value, spec);
  } else {
    app.addTarget(spec);
  }
  showDialog.value = false;
  $q.notify({
    message: isEditing.value ? 'Target updated' : 'Target created',
    color: 'positive',
    position: 'bottom-right',
    timeout: 2000,
  });
}

function remove(name: string) {
  $q.dialog({
    title: 'Remove Target',
    message: `Are you sure you want to remove target "${name}"?`,
    cancel: true,
  }).onOk(() => {
    app.removeTarget(name);
  });
}
</script>

<template>
  <q-page class="q-pa-md">
    <div class="row items-center q-mb-md">
      <div class="text-h5 col">Targets</div>
      <q-btn color="primary" icon="add" label="Add Target" @click="openCreate" />
    </div>

    <q-banner v-if="targets.length === 0" class="bg-grey-3 q-mb-md" rounded>
      <template #avatar>
        <q-icon name="info" color="grey" />
      </template>
      No targets configured. Click "Add Target" to create one, or import from the Dashboard.
    </q-banner>

    <q-table
      v-else
      :rows="targets"
      :columns="[
        { name: 'name', label: 'Name', field: 'name', align: 'left', sortable: true },
        { name: 'process', label: 'Process', field: 'process', align: 'left' },
        { name: 'windowClass', label: 'Window Class', field: 'windowClass', align: 'left' },
        { name: 'windowTitle', label: 'Window Title', field: 'windowTitle', align: 'left' },
        { name: 'matchMode', label: 'Match Mode', field: 'matchMode', align: 'center' },
        { name: 'actions', label: 'Actions', field: 'name', align: 'center' },
      ]"
      row-key="name"
      flat
      bordered
    >
      <template #body-cell-actions="props">
        <q-td :props="props">
          <q-btn flat round icon="edit" color="primary" @click="openEdit(props.value as string)">
            <q-tooltip>Edit target</q-tooltip>
          </q-btn>
          <q-btn flat round icon="delete" color="negative" @click="remove(props.value as string)">
            <q-tooltip>Remove target</q-tooltip>
          </q-btn>
        </q-td>
      </template>
    </q-table>

    <!-- Create / Edit dialog -->
    <q-dialog v-model="showDialog" persistent>
      <q-card style="min-width: 480px">
        <q-card-section>
          <div class="text-h6">{{ isEditing ? 'Edit Target' : 'Create Target' }}</div>
        </q-card-section>

        <q-card-section class="column q-gutter-sm">
          <q-input v-model="form.name" label="Name *" outlined dense :disable="isEditing" />
          <q-input v-model="form.process_name" label="Process Name" outlined dense hint="e.g. notepad.exe" />
          <q-input v-model="form.window_class" label="Window Class" outlined dense hint="e.g. Notepad" />
          <q-input v-model="form.window_title" label="Window Title" outlined dense />
          <q-select
            v-model="form.title_match_mode"
            :options="matchModeOptions"
            label="Title Match Mode"
            outlined
            dense
            emit-value
            map-options
          />
          <div class="row q-gutter-md">
            <q-toggle v-model="form.top_level_only" label="Top-level only" />
            <q-toggle v-model="form.visible_only" label="Visible only" />
          </div>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat label="Cancel" v-close-popup />
          <q-btn color="primary" :label="isEditing ? 'Save' : 'Create'" @click="saveTarget" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>
