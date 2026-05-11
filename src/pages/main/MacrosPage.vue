<script setup lang="ts">
import { computed, reactive, ref } from 'vue';
import { useAppStore } from 'stores/app';
import { useQuasar } from 'quasar';
import type { MacroSequence, MacroStep } from 'src/types';

const $q = useQuasar();
const app = useAppStore();

const macros = computed(() => {
  return Object.entries(app.macros).map(([name, m]) => ({
    name,
    target: m.target_name,
    dispatch: m.dispatch_mode,
    steps: m.steps.length,
    hasPhases: m.has_phases,
  }));
});

const targetOptions = computed(() => Object.keys(app.targets));

const dispatchOptions = [
  { label: 'SendInput', value: 'SendInput' as const },
  { label: 'WindowMessage', value: 'WindowMessage' as const },
  { label: 'Logitech', value: 'Logitech' as const },
];

// -- Dialog state -----------------------------------------------------------
const showDialog = ref(false);
const isEditing = ref(false);
const editingOriginalName = ref('');

const form: {
  name: string;
  target_name: string;
  dispatch_mode: MacroSequence['dispatch_mode'];
  source_file: string;
} = reactive({
  name: '',
  target_name: '',
  dispatch_mode: 'SendInput',
  source_file: '',
});

// -- Step editor state ------------------------------------------------------
const showStepDialog = ref(false);
const editingMacroName = ref('');

const stepTypeOptions = [
  { label: 'Delay', value: 'Delay' },
  { label: 'Key', value: 'Key' },
  { label: 'Text', value: 'Text' },
  { label: 'Mouse Move', value: 'MouseMove' },
  { label: 'Mouse Click', value: 'MouseClick' },
];

const keyActionOptions = ['Tap', 'Down', 'Up'] as const;
const coordModeOptions = ['Screen', 'Client'] as const;
const mouseButtonOptions = ['Left', 'Right', 'Middle'] as const;
const mouseActionOptions = ['Click', 'Down', 'Up'] as const;

const newStepType = ref('Delay');
const newStepDelay = ref(100);
const newStepKeyVk = ref(0);
const newStepKeyAction = ref<'Tap' | 'Down' | 'Up'>('Tap');
const newStepText = ref('');
const newStepMouseX = ref(0);
const newStepMouseY = ref(0);
const newStepCoordMode = ref<'Screen' | 'Client'>('Screen');
const newStepMouseButton = ref<'Left' | 'Right' | 'Middle'>('Left');
const newStepMouseAction = ref<'Click' | 'Down' | 'Up'>('Click');

const editingSteps = ref<MacroStep[]>([]);

function resetForm() {
  form.name = '';
  form.target_name = '';
  form.dispatch_mode = 'SendInput';
  form.source_file = '';
}

function openCreate() {
  resetForm();
  isEditing.value = false;
  showDialog.value = true;
}

function openEdit(name: string) {
  const m = app.macros[name];
  if (!m) return;
  form.name = m.name;
  form.target_name = m.target_name;
  form.dispatch_mode = m.dispatch_mode;
  form.source_file = m.source_file;
  editingOriginalName.value = name;
  isEditing.value = true;
  showDialog.value = true;
}

function saveMacro() {
  if (!form.name.trim()) {
    $q.notify({ message: 'Name is required', color: 'negative', position: 'bottom-right' });
    return;
  }
  if (!isEditing.value && app.macros[form.name]) {
    $q.notify({ message: `Macro "${form.name}" already exists`, color: 'negative', position: 'bottom-right' });
    return;
  }
  if (isEditing.value && form.name !== editingOriginalName.value && app.macros[form.name]) {
    $q.notify({ message: `Macro "${form.name}" already exists`, color: 'negative', position: 'bottom-right' });
    return;
  }

  const existing = isEditing.value ? app.macros[editingOriginalName.value] : undefined;

  const seq: MacroSequence = {
    name: form.name,
    target_name: form.target_name,
    dispatch_mode: form.dispatch_mode,
    source_file: form.source_file,
    steps: existing?.steps ?? [],
    on_press_steps: existing?.on_press_steps ?? [],
    on_hold_steps: existing?.on_hold_steps ?? [],
    on_release_steps: existing?.on_release_steps ?? [],
    has_phases: existing?.has_phases ?? false,
  };

  if (isEditing.value) {
    app.updateMacro(editingOriginalName.value, seq);
  } else {
    app.addMacro(seq);
  }
  showDialog.value = false;
  $q.notify({
    message: isEditing.value ? 'Macro updated' : 'Macro created',
    color: 'positive',
    position: 'bottom-right',
    timeout: 2000,
  });
}

// -- Step editing -----------------------------------------------------------
function openStepEditor(name: string) {
  const m = app.macros[name];
  if (!m) return;
  editingMacroName.value = name;
  editingSteps.value = [...m.steps];
  showStepDialog.value = true;
}

function stepLabel(step: MacroStep): string {
  if ('Delay' in step) return `Delay ${String(step.Delay.milliseconds)}ms`;
  if ('Key' in step) return `Key VK:${String(step.Key.virtual_key)} ${step.Key.action}`;
  if ('Text' in step) return `Text "${step.Text.text}"`;
  if ('MouseMove' in step) return `MouseMove (${String(step.MouseMove.x)}, ${String(step.MouseMove.y)}) ${step.MouseMove.coordinate_mode}`;
  if ('MouseClick' in step) return `Click ${step.MouseClick.button} ${step.MouseClick.action}`;
  return 'Unknown';
}

function addStep() {
  let step: MacroStep;
  switch (newStepType.value) {
    case 'Key':
      step = { Key: { virtual_key: newStepKeyVk.value, action: newStepKeyAction.value } };
      break;
    case 'Text':
      step = { Text: { text: newStepText.value } };
      break;
    case 'MouseMove':
      step = { MouseMove: { x: newStepMouseX.value, y: newStepMouseY.value, coordinate_mode: newStepCoordMode.value } };
      break;
    case 'MouseClick':
      step = { MouseClick: { button: newStepMouseButton.value, action: newStepMouseAction.value } };
      break;
    default:
      step = { Delay: { milliseconds: newStepDelay.value } };
  }
  editingSteps.value.push(step);
}

function removeStep(index: number) {
  editingSteps.value.splice(index, 1);
}

function moveStepUp(index: number) {
  if (index <= 0) return;
  const arr = editingSteps.value;
  [arr[index - 1], arr[index]] = [arr[index]!, arr[index - 1]!];
}

function moveStepDown(index: number) {
  const arr = editingSteps.value;
  if (index >= arr.length - 1) return;
  [arr[index], arr[index + 1]] = [arr[index + 1]!, arr[index]!];
}

function saveSteps() {
  const m = app.macros[editingMacroName.value];
  if (!m) return;
  const updated: MacroSequence = { ...m, steps: [...editingSteps.value] };
  app.updateMacro(editingMacroName.value, updated);
  showStepDialog.value = false;
  $q.notify({ message: 'Steps updated', color: 'positive', position: 'bottom-right', timeout: 2000 });
}

// -- Play / Remove ----------------------------------------------------------
async function play(name: string) {
  const ok = await app.playMacro(name);
  $q.notify({
    message: ok ? `Macro "${name}" executed` : `Macro "${name}" failed`,
    color: ok ? 'positive' : 'negative',
    position: 'bottom-right',
    timeout: 2000,
  });
}

function remove(name: string) {
  $q.dialog({
    title: 'Remove Macro',
    message: `Are you sure you want to remove macro "${name}"?`,
    cancel: true,
  }).onOk(() => {
    app.removeMacro(name);
  });
}
</script>

<template>
  <q-page class="q-pa-md">
    <div class="row items-center q-mb-md">
      <div class="text-h5 col">Macros</div>
      <q-btn color="primary" icon="add" label="Add Macro" @click="openCreate" />
    </div>

    <q-banner v-if="macros.length === 0" class="bg-grey-3 q-mb-md" rounded>
      <template #avatar>
        <q-icon name="info" color="grey" />
      </template>
      No macros configured. Click "Add Macro" to create one, or import from the Dashboard.
    </q-banner>

    <q-table
      v-else
      :rows="macros"
      :columns="[
        { name: 'name', label: 'Name', field: 'name', align: 'left', sortable: true },
        { name: 'target', label: 'Target', field: 'target', align: 'left' },
        { name: 'dispatch', label: 'Dispatch', field: 'dispatch', align: 'left' },
        { name: 'steps', label: 'Steps', field: 'steps', align: 'center' },
        { name: 'phases', label: 'Phased', field: 'hasPhases', align: 'center' },
        { name: 'actions', label: 'Actions', field: 'name', align: 'center' },
      ]"
      row-key="name"
      flat
      bordered
    >
      <template #body-cell-phases="props">
        <q-td :props="props">
          <q-icon v-if="props.value" name="check_circle" color="positive" />
          <q-icon v-else name="remove" color="grey" />
        </q-td>
      </template>
      <template #body-cell-actions="props">
        <q-td :props="props">
          <q-btn flat round icon="play_arrow" color="primary" @click="play(props.value as string)">
            <q-tooltip>Play macro</q-tooltip>
          </q-btn>
          <q-btn flat round icon="list" color="secondary" @click="openStepEditor(props.value as string)">
            <q-tooltip>Edit steps</q-tooltip>
          </q-btn>
          <q-btn flat round icon="edit" color="primary" @click="openEdit(props.value as string)">
            <q-tooltip>Edit macro</q-tooltip>
          </q-btn>
          <q-btn flat round icon="delete" color="negative" @click="remove(props.value as string)">
            <q-tooltip>Remove macro</q-tooltip>
          </q-btn>
        </q-td>
      </template>
    </q-table>

    <!-- Create / Edit macro dialog -->
    <q-dialog v-model="showDialog" persistent>
      <q-card style="min-width: 480px">
        <q-card-section>
          <div class="text-h6">{{ isEditing ? 'Edit Macro' : 'Create Macro' }}</div>
        </q-card-section>

        <q-card-section class="column q-gutter-sm">
          <q-input v-model="form.name" label="Name *" outlined dense />
          <q-select
            v-model="form.target_name"
            :options="targetOptions"
            label="Target"
            outlined
            dense
            clearable
            use-input
            new-value-mode="add-unique"
            hint="Select an existing target or type a new name"
          />
          <q-select
            v-model="form.dispatch_mode"
            :options="dispatchOptions"
            label="Dispatch Mode"
            outlined
            dense
            emit-value
            map-options
          />
          <q-input v-model="form.source_file" label="Source File (optional)" outlined dense hint="Path to .macro file" />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat label="Cancel" v-close-popup />
          <q-btn color="primary" :label="isEditing ? 'Save' : 'Create'" @click="saveMacro" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- Step editor dialog -->
    <q-dialog v-model="showStepDialog" persistent maximized transition-show="slide-up" transition-hide="slide-down">
      <q-card>
        <q-bar class="bg-primary text-white">
          <div>Edit Steps — {{ editingMacroName }}</div>
          <q-space />
          <q-btn dense flat icon="close" @click="showStepDialog = false" />
        </q-bar>

        <q-card-section class="row q-gutter-md" style="max-height: 80vh; overflow: auto">
          <!-- Step list -->
          <div class="col">
            <div class="text-subtitle1 q-mb-sm">Steps ({{ editingSteps.length }})</div>
            <q-list bordered separator>
              <q-item v-for="(step, idx) in editingSteps" :key="idx">
                <q-item-section avatar>
                  <q-badge color="grey" :label="String(idx + 1)" />
                </q-item-section>
                <q-item-section>
                  <q-item-label>{{ stepLabel(step) }}</q-item-label>
                </q-item-section>
                <q-item-section side>
                  <div class="row q-gutter-xs">
                    <q-btn flat round dense icon="arrow_upward" size="sm" :disable="idx === 0" @click="moveStepUp(idx)" />
                    <q-btn flat round dense icon="arrow_downward" size="sm" :disable="idx === editingSteps.length - 1" @click="moveStepDown(idx)" />
                    <q-btn flat round dense icon="delete" size="sm" color="negative" @click="removeStep(idx)" />
                  </div>
                </q-item-section>
              </q-item>
              <q-item v-if="editingSteps.length === 0">
                <q-item-section class="text-grey text-center">
                  No steps yet. Use the form on the right to add steps.
                </q-item-section>
              </q-item>
            </q-list>
          </div>

          <!-- Add step form -->
          <q-card flat bordered class="col-4">
            <q-card-section>
              <div class="text-subtitle1">Add Step</div>
            </q-card-section>
            <q-card-section class="column q-gutter-sm">
              <q-select v-model="newStepType" :options="stepTypeOptions" label="Type" outlined dense emit-value map-options />

              <!-- Delay -->
              <q-input v-if="newStepType === 'Delay'" v-model.number="newStepDelay" label="Milliseconds" outlined dense type="number" />

              <!-- Key -->
              <template v-if="newStepType === 'Key'">
                <q-input v-model.number="newStepKeyVk" label="Virtual Key Code" outlined dense type="number" hint="e.g. 65 for 'A', 13 for Enter" />
                <q-select v-model="newStepKeyAction" :options="[...keyActionOptions]" label="Action" outlined dense />
              </template>

              <!-- Text -->
              <q-input v-if="newStepType === 'Text'" v-model="newStepText" label="Text" outlined dense autogrow />

              <!-- MouseMove -->
              <template v-if="newStepType === 'MouseMove'">
                <q-input v-model.number="newStepMouseX" label="X" outlined dense type="number" />
                <q-input v-model.number="newStepMouseY" label="Y" outlined dense type="number" />
                <q-select v-model="newStepCoordMode" :options="[...coordModeOptions]" label="Coordinate Mode" outlined dense />
              </template>

              <!-- MouseClick -->
              <template v-if="newStepType === 'MouseClick'">
                <q-select v-model="newStepMouseButton" :options="[...mouseButtonOptions]" label="Button" outlined dense />
                <q-select v-model="newStepMouseAction" :options="[...mouseActionOptions]" label="Action" outlined dense />
              </template>

              <q-btn color="primary" icon="add" label="Add Step" @click="addStep" class="q-mt-sm" />
            </q-card-section>
          </q-card>
        </q-card-section>

        <q-card-actions align="right" class="q-pa-md">
          <q-btn flat label="Cancel" @click="showStepDialog = false" />
          <q-btn color="primary" label="Save Steps" icon="save" @click="saveSteps" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>
