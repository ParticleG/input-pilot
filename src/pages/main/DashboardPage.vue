<script setup lang="ts">
import { useAppStore } from 'stores/app';
import { useQuasar } from 'quasar';
import { ref, onMounted } from 'vue';

const $q = useQuasar();
const app = useAppStore();

const importPath = ref('config/app.ini');
const showImportDialog = ref(false);

onMounted(async () => {
  await app.refreshDaemonStatus();
});

async function importConfig() {
  try {
    await app.importFromFile(importPath.value);
    showImportDialog.value = false;
    $q.notify({
      message: 'Config imported',
      color: 'positive',
      position: 'bottom-right',
      timeout: 2000,
    });
  } catch (e) {
    $q.notify({
      message: `Import failed: ${String(e)}`,
      color: 'negative',
      position: 'bottom-right',
      timeout: 4000,
    });
  }
}

async function playMacro(name: string) {
  const ok = await app.playMacro(name);
  $q.notify({
    message: ok ? `Macro "${name}" executed` : `Macro "${name}" failed`,
    color: ok ? 'positive' : 'negative',
    position: 'bottom-right',
    timeout: 2000,
  });
}

async function toggleDaemon() {
  if (app.daemonRunning) {
    await app.stopDaemon();
    $q.notify({
      message: 'Hotkey daemon stopped',
      color: 'warning',
      position: 'bottom-right',
      timeout: 2000,
    });
  } else {
    const ok = await app.startDaemon();
    $q.notify({
      message: ok ? 'Hotkey daemon started' : 'Failed to start daemon',
      color: ok ? 'positive' : 'negative',
      position: 'bottom-right',
      timeout: 2000,
    });
  }
}
</script>

<template>
  <q-page class="q-pa-md column q-gutter-y-md">
    <!-- Hotkey Daemon -->
    <q-card flat bordered>
      <q-card-section>
        <div class="row items-center q-gutter-sm">
          <div class="text-h6 col">Hotkey Daemon</div>
          <q-chip
            :color="app.daemonRunning ? 'positive' : 'grey'"
            text-color="white"
            dense
            :label="app.daemonRunning ? 'Running' : 'Stopped'"
          />
          <q-btn
            :color="app.daemonRunning ? 'negative' : 'positive'"
            :icon="app.daemonRunning ? 'stop' : 'play_arrow'"
            :label="app.daemonRunning ? 'Stop' : 'Start'"
            flat
            @click="toggleDaemon"
          />
        </div>
        <div class="text-caption text-grey q-mt-xs">
          When running, registered hotkeys are active system-wide even if the window is hidden.
        </div>
      </q-card-section>
    </q-card>

    <!-- Config toolbar -->
    <q-card flat bordered>
      <q-card-section>
        <div class="row items-center q-gutter-sm">
          <div class="text-h6 col">Configuration</div>
          <q-spinner-dots v-if="app.syncing" color="primary" size="20px" />
          <q-btn flat icon="file_upload" label="Import INI" @click="showImportDialog = true" />
        </div>
      </q-card-section>
    </q-card>

    <!-- Stats -->
    <div class="row q-gutter-md">
      <q-card flat bordered class="col">
        <q-card-section class="text-center">
          <div class="text-h4 text-primary">{{ app.targetCount }}</div>
          <div class="text-subtitle2 text-grey">Targets</div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col">
        <q-card-section class="text-center">
          <div class="text-h4 text-primary">{{ app.macroCount }}</div>
          <div class="text-subtitle2 text-grey">Macros</div>
        </q-card-section>
      </q-card>
      <q-card flat bordered class="col">
        <q-card-section class="text-center">
          <div class="text-h4 text-primary">{{ app.hotkeyCount }}</div>
          <div class="text-subtitle2 text-grey">Hotkeys</div>
        </q-card-section>
      </q-card>
    </div>

    <!-- Quick Play -->
    <q-card v-if="app.macroCount > 0" flat bordered>
      <q-card-section>
        <div class="text-h6">Quick Play</div>
      </q-card-section>
      <q-card-section>
        <div class="row q-gutter-sm">
          <q-btn
            v-for="(_, name) in app.macros"
            :key="name"
            color="primary"
            outline
            :label="name"
            icon="play_arrow"
            @click="playMacro(name)"
          />
        </div>
      </q-card-section>
    </q-card>

    <!-- Hotkeys -->
    <q-card v-if="app.hotkeyCount > 0" flat bordered>
      <q-card-section>
        <div class="text-h6">Hotkeys</div>
      </q-card-section>
      <q-list separator>
        <q-item v-for="hk in app.hotkeys" :key="hk.id">
          <q-item-section avatar>
            <q-icon
              :name="app.hotkeyStates[hk.id] ? 'play_circle' : 'keyboard'"
              :color="app.hotkeyStates[hk.id] ? 'positive' : undefined"
            />
          </q-item-section>
          <q-item-section>
            <q-item-label>{{ hk.description }}</q-item-label>
            <q-item-label caption>
              {{ hk.action }} · {{ hk.trigger_mode }}
              <template v-if="hk.macro_name"> · macro: {{ hk.macro_name }}</template>
            </q-item-label>
          </q-item-section>
          <q-item-section side>
            <q-chip
              dense
              :color="app.hotkeyStates[hk.id] ? 'positive' : 'primary'"
              text-color="white"
            >
              {{ app.hotkeyStates[hk.id] ? 'Active' : hk.trigger_mode }}
            </q-chip>
          </q-item-section>
        </q-item>
      </q-list>
    </q-card>

    <!-- Empty state -->
    <q-card v-if="app.targetCount === 0 && app.macroCount === 0" flat bordered>
      <q-card-section class="text-center q-pa-xl">
        <q-icon name="info" size="48px" color="grey" />
        <div class="text-h6 text-grey q-mt-md">No configuration loaded</div>
        <div class="text-body2 text-grey q-mb-md">
          Import from an INI file or add targets and macros manually.
        </div>
        <q-btn
          color="primary"
          icon="file_upload"
          label="Import INI File"
          @click="showImportDialog = true"
        />
      </q-card-section>
    </q-card>

    <!-- Import dialog -->
    <q-dialog v-model="showImportDialog">
      <q-card style="min-width: 400px">
        <q-card-section>
          <div class="text-h6">Import Configuration</div>
        </q-card-section>
        <q-card-section>
          <q-input
            v-model="importPath"
            label="INI File Path"
            outlined
            dense
            hint="Path relative to the executable, e.g. config/app.ini"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat label="Cancel" v-close-popup />
          <q-btn color="primary" label="Import" :loading="app.loading" @click="importConfig" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<style scoped></style>
