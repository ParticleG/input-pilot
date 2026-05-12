<script setup lang="ts">
import { computed, reactive, ref } from 'vue';
import { useAppStore } from 'stores/app';
import { useQuasar } from 'quasar';
import type { HotkeyBinding } from 'src/types';

const $q = useQuasar();
const app = useAppStore();

// -- Modifier constants -----------------------------------------------------
const MOD_ALT = 0x0001;
const MOD_CONTROL = 0x0002;
const MOD_SHIFT = 0x0004;
const MOD_WIN = 0x0008;

// -- VK code display helper -------------------------------------------------
function vkName(vk: number): string {
  if (vk >= 0x70 && vk <= 0x87) return `F${vk - 0x70 + 1}`;
  if (vk >= 0x30 && vk <= 0x39) return String(vk - 0x30);
  if (vk >= 0x41 && vk <= 0x5a) return String.fromCharCode(vk);
  const named: Record<number, string> = {
    0x08: 'Backspace', 0x09: 'Tab', 0x0d: 'Enter', 0x10: 'Shift',
    0x11: 'Ctrl', 0x12: 'Alt', 0x13: 'Pause', 0x14: 'CapsLock',
    0x1b: 'Esc', 0x20: 'Space', 0x21: 'PgUp', 0x22: 'PgDn',
    0x23: 'End', 0x24: 'Home', 0x25: 'Left', 0x26: 'Up',
    0x27: 'Right', 0x28: 'Down', 0x2c: 'PrtSc', 0x2d: 'Ins',
    0x2e: 'Del', 0x5b: 'LWin', 0x5c: 'RWin', 0x5d: 'Menu',
    0x6f: 'Num/', 0x6a: 'Num*', 0x6d: 'Num-', 0x6b: 'Num+',
    0x6e: 'Num.', 0x90: 'NumLock', 0x91: 'ScrollLock',
  };
  return named[vk] ?? `VK:0x${vk.toString(16).toUpperCase().padStart(2, '0')}`;
}

function keyComboLabel(modifiers: number, vk: number): string {
  const parts: string[] = [];
  if (modifiers & MOD_CONTROL) parts.push('Ctrl');
  if (modifiers & MOD_ALT) parts.push('Alt');
  if (modifiers & MOD_SHIFT) parts.push('Shift');
  if (modifiers & MOD_WIN) parts.push('Win');
  if (vk !== 0) parts.push(vkName(vk));
  return parts.join('+') || '(none)';
}

// -- Table rows -------------------------------------------------------------
const rows = computed(() => app.hotkeys);

const macroOptions = computed(() => Object.keys(app.macros));
const targetOptions = computed(() => Object.keys(app.targets));

const actionOptions = [
  { label: 'Play Macro', value: 'PlayMacro' as const },
  { label: 'Play File', value: 'PlayFile' as const },
  { label: 'Record Toggle', value: 'RecordToggle' as const },
];

const triggerModeOptions = [
  { label: 'Once', value: 'Once' as const },
  { label: 'Toggle', value: 'Toggle' as const },
  { label: 'Hold', value: 'Hold' as const },
  { label: 'Phased', value: 'Phased' as const },
];

const dispatchOptions = [
  { label: 'SendInput', value: 'SendInput' as const },
  { label: 'WindowMessage', value: 'WindowMessage' as const },
  { label: 'Logitech', value: 'Logitech' as const },
];

// -- Dialog state -----------------------------------------------------------
const showDialog = ref(false);
const isEditing = ref(false);
const editingId = ref<number | null>(null);

interface FormState {
  description: string;
  mod_ctrl: boolean;
  mod_alt: boolean;
  mod_shift: boolean;
  mod_win: boolean;
  virtual_key: number;
  action: HotkeyBinding['action'];
  trigger_mode: HotkeyBinding['trigger_mode'];
  repeat_delay_ms: number;
  macro_name: string;
  file_path: string;
  target_name: string;
  dispatch_mode: HotkeyBinding['dispatch_mode'];
  has_dispatch_override: boolean;
}

const form: FormState = reactive({
  description: '',
  mod_ctrl: false,
  mod_alt: false,
  mod_shift: false,
  mod_win: false,
  virtual_key: 0,
  action: 'PlayMacro',
  trigger_mode: 'Once',
  repeat_delay_ms: 500,
  macro_name: '',
  file_path: '',
  target_name: '',
  dispatch_mode: 'SendInput',
  has_dispatch_override: false,
});

const showRepeatDelay = computed(() =>
  form.trigger_mode === 'Toggle' || form.trigger_mode === 'Hold' || form.trigger_mode === 'Phased',
);

function resetForm() {
  form.description = '';
  form.mod_ctrl = false;
  form.mod_alt = false;
  form.mod_shift = false;
  form.mod_win = false;
  form.virtual_key = 0;
  form.action = 'PlayMacro';
  form.trigger_mode = 'Once';
  form.repeat_delay_ms = 500;
  form.macro_name = '';
  form.file_path = '';
  form.target_name = '';
  form.dispatch_mode = 'SendInput';
  form.has_dispatch_override = false;
}

function modifiersToForm(modifiers: number) {
  form.mod_ctrl = !!(modifiers & MOD_CONTROL);
  form.mod_alt = !!(modifiers & MOD_ALT);
  form.mod_shift = !!(modifiers & MOD_SHIFT);
  form.mod_win = !!(modifiers & MOD_WIN);
}

function formToModifiers(): number {
  let m = 0;
  if (form.mod_ctrl) m |= MOD_CONTROL;
  if (form.mod_alt) m |= MOD_ALT;
  if (form.mod_shift) m |= MOD_SHIFT;
  if (form.mod_win) m |= MOD_WIN;
  return m;
}

function openCreate() {
  resetForm();
  isEditing.value = false;
  editingId.value = null;
  showDialog.value = true;
}

function openEdit(hotkey: HotkeyBinding) {
  form.description = hotkey.description;
  modifiersToForm(hotkey.modifiers);
  form.virtual_key = hotkey.virtual_key;
  form.action = hotkey.action;
  form.trigger_mode = hotkey.trigger_mode;
  form.repeat_delay_ms = hotkey.repeat_delay_ms;
  form.macro_name = hotkey.macro_name;
  form.file_path = hotkey.file_path;
  form.target_name = hotkey.target_name;
  form.dispatch_mode = hotkey.dispatch_mode;
  form.has_dispatch_override = hotkey.has_dispatch_override;
  editingId.value = hotkey.id;
  isEditing.value = true;
  showDialog.value = true;
}

function nextId(): number {
  if (app.hotkeys.length === 0) return 1;
  return Math.max(...app.hotkeys.map((h) => h.id)) + 1;
}

function saveHotkey() {
  if (form.virtual_key <= 0) {
    $q.notify({ message: 'Virtual key must be greater than 0', color: 'negative', position: 'bottom-right' });
    return;
  }

  const binding: HotkeyBinding = {
    id: isEditing.value && editingId.value !== null ? editingId.value : nextId(),
    modifiers: formToModifiers(),
    virtual_key: form.virtual_key,
    action: form.action,
    trigger_mode: form.trigger_mode,
    repeat_delay_ms: form.repeat_delay_ms,
    macro_name: form.action === 'PlayMacro' ? form.macro_name : '',
    file_path: form.action === 'PlayFile' ? form.file_path : '',
    target_name: form.target_name,
    dispatch_mode: form.dispatch_mode,
    has_dispatch_override: form.has_dispatch_override,
    description: form.description,
  };

  if (isEditing.value && editingId.value !== null) {
    const idx = app.hotkeys.findIndex((h) => h.id === editingId.value);
    if (idx !== -1) {
      app.hotkeys.splice(idx, 1, binding);
    }
  } else {
    app.hotkeys.push(binding);
  }

  showDialog.value = false;
  $q.notify({
    message: isEditing.value ? 'Hotkey updated' : 'Hotkey created',
    color: 'positive',
    position: 'bottom-right',
    timeout: 2000,
  });
}

function remove(hotkey: HotkeyBinding) {
  const label = hotkey.description || keyComboLabel(hotkey.modifiers, hotkey.virtual_key);
  $q.dialog({
    title: 'Remove Hotkey',
    message: `Are you sure you want to remove hotkey "${label}"?`,
    cancel: true,
  }).onOk(() => {
    const idx = app.hotkeys.findIndex((h) => h.id === hotkey.id);
    if (idx !== -1) app.hotkeys.splice(idx, 1);
  });
}
</script>

<template>
  <q-page class="q-pa-md">
    <div class="row items-center q-mb-md">
      <div class="text-h5 col">Hotkeys</div>
      <q-btn color="primary" icon="add" label="Add Hotkey" @click="openCreate" />
    </div>

    <q-banner v-if="rows.length === 0" class="bg-grey-3 q-mb-md" rounded>
      <template #avatar>
        <q-icon name="info" color="grey" />
      </template>
      No hotkeys configured. Click "Add Hotkey" to create one.
    </q-banner>

    <q-table
      v-else
      :rows="rows"
      :columns="[
        { name: 'status', label: 'Status', field: 'id', align: 'center', style: 'width: 60px' },
        { name: 'description', label: 'Description', field: 'description', align: 'left', sortable: true },
        { name: 'keys', label: 'Keys', field: (row: HotkeyBinding) => keyComboLabel(row.modifiers, row.virtual_key), align: 'left' },
        { name: 'action', label: 'Action', field: 'action', align: 'left', sortable: true },
        { name: 'trigger_mode', label: 'Trigger Mode', field: 'trigger_mode', align: 'left' },
        { name: 'macro_file', label: 'Macro / File', field: (row: HotkeyBinding) => row.action === 'PlayMacro' ? row.macro_name : row.action === 'PlayFile' ? row.file_path : '—', align: 'left' },
        { name: 'target', label: 'Target', field: 'target_name', align: 'left' },
        { name: 'actions', label: 'Actions', field: 'id', align: 'center' },
      ]"
      row-key="id"
      flat
      bordered
    >
      <template #body-cell-status="props">
        <q-td :props="props">
          <q-icon
            :name="app.hotkeyStates[(props.row as HotkeyBinding).id] ? 'fiber_manual_record' : 'radio_button_unchecked'"
            :color="app.hotkeyStates[(props.row as HotkeyBinding).id] ? 'positive' : 'grey'"
            size="sm"
          >
            <q-tooltip>
              {{ app.hotkeyStates[(props.row as HotkeyBinding).id] ? 'Active' : 'Idle' }}
            </q-tooltip>
          </q-icon>
        </q-td>
      </template>
      <template #body-cell-actions="props">
        <q-td :props="props">
          <q-btn flat round icon="edit" color="primary" @click="openEdit(props.row as HotkeyBinding)">
            <q-tooltip>Edit hotkey</q-tooltip>
          </q-btn>
          <q-btn flat round icon="delete" color="negative" @click="remove(props.row as HotkeyBinding)">
            <q-tooltip>Remove hotkey</q-tooltip>
          </q-btn>
        </q-td>
      </template>
    </q-table>

    <!-- Create / Edit hotkey dialog -->
    <q-dialog v-model="showDialog" persistent>
      <q-card style="min-width: 480px">
        <q-card-section>
          <div class="text-h6">{{ isEditing ? 'Edit Hotkey' : 'Create Hotkey' }}</div>
        </q-card-section>

        <q-card-section class="column q-gutter-sm">
          <q-input v-model="form.description" label="Description" outlined dense />

          <!-- Modifier checkboxes -->
          <div>
            <div class="text-caption text-grey q-mb-xs">Modifiers</div>
            <div class="row q-gutter-md">
              <q-checkbox v-model="form.mod_ctrl" label="Ctrl" />
              <q-checkbox v-model="form.mod_alt" label="Alt" />
              <q-checkbox v-model="form.mod_shift" label="Shift" />
              <q-checkbox v-model="form.mod_win" label="Win" />
            </div>
          </div>

          <q-input
            v-model.number="form.virtual_key"
            label="Virtual Key Code *"
            outlined
            dense
            type="number"
            hint="e.g. 0x70=F1, 0x41=A, 0x31=1, 0x0D=Enter"
          />

          <q-select
            v-model="form.action"
            :options="actionOptions"
            label="Action"
            outlined
            dense
            emit-value
            map-options
          />

          <q-select
            v-if="form.action === 'PlayMacro'"
            v-model="form.macro_name"
            :options="macroOptions"
            label="Macro Name"
            outlined
            dense
            clearable
            use-input
            new-value-mode="add-unique"
            hint="Select a configured macro"
          />

          <q-input
            v-if="form.action === 'PlayFile'"
            v-model="form.file_path"
            label="File Path"
            outlined
            dense
            hint="Path to .macro file"
          />

          <q-select
            v-model="form.trigger_mode"
            :options="triggerModeOptions"
            label="Trigger Mode"
            outlined
            dense
            emit-value
            map-options
          />

          <q-input
            v-if="showRepeatDelay"
            v-model.number="form.repeat_delay_ms"
            label="Repeat Delay (ms)"
            outlined
            dense
            type="number"
            hint="Delay between repeats in milliseconds"
          />

          <q-select
            v-model="form.target_name"
            :options="targetOptions"
            label="Target (optional)"
            outlined
            dense
            clearable
            use-input
            new-value-mode="add-unique"
            hint="Override target for this hotkey"
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

          <q-toggle v-model="form.has_dispatch_override" label="Override dispatch mode" />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat label="Cancel" v-close-popup />
          <q-btn color="primary" :label="isEditing ? 'Save' : 'Create'" @click="saveHotkey" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>
