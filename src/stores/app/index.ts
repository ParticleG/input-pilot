import { defineStore, acceptHMRUpdate } from 'pinia';
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Notify } from 'quasar';
import type { AppConfig, MacroSequence, TargetSpec, HotkeyBinding, HotkeyStateEvent, WindowMatch } from 'src/types';

export const useAppStore = defineStore(
  'app',
  () => {
    // -----------------------------------------------------------------------
    // Persistent config state (saved to localStorage via pinia-plugin-persistedstate)
    // -----------------------------------------------------------------------
    const targets = ref<Record<string, TargetSpec>>({});
    const macros = ref<Record<string, MacroSequence>>({});
    const hotkeys = ref<HotkeyBinding[]>([]);
    const recordingsDirectory = ref('recordings');

    // -----------------------------------------------------------------------
    // Transient UI state (not persisted)
    // -----------------------------------------------------------------------
    const windows = ref<WindowMatch[]>([]);
    const loading = ref(false);
    const error = ref<string | null>(null);
    const daemonRunning = ref(false);

    /** Reactive map of hotkey id → active state (for Toggle/Hold/Phased indicators) */
    const hotkeyStates = ref<Record<number, boolean>>({});

    // -----------------------------------------------------------------------
    // Computed
    // -----------------------------------------------------------------------
    const targetCount = computed(() => Object.keys(targets.value).length);
    const macroCount = computed(() => Object.keys(macros.value).length);
    const hotkeyCount = computed(() => hotkeys.value.length);

    const config = computed<AppConfig>(() => ({
      targets: targets.value,
      macros: macros.value,
      hotkeys: hotkeys.value,
      recordings_directory: recordingsDirectory.value,
    }));

    // -----------------------------------------------------------------------
    // Auto-sync: watch config and push to Rust backend on change
    // -----------------------------------------------------------------------
    let syncTimer: ReturnType<typeof setTimeout> | null = null;
    const syncing = ref(false);

    function scheduleSyncToBackend() {
      if (syncTimer) clearTimeout(syncTimer);
      syncTimer = setTimeout(() => {
        void doSync();
      }, 300);
    }

    async function doSync() {
      syncing.value = true;
      try {
        await invoke('apply_config', { config: config.value });
      } catch (e) {
        error.value = String(e);
      } finally {
        syncing.value = false;
      }
    }

    // Deep-watch all persisted config refs; auto-sync after 300ms debounce
    watch(
      [targets, macros, hotkeys, recordingsDirectory],
      () => {
        scheduleSyncToBackend();
      },
      { deep: true },
    );

    // -----------------------------------------------------------------------
    // Config management
    // -----------------------------------------------------------------------

    /** Import config from an INI file into Pinia state (auto-syncs via watch) */
    async function importFromFile(path: string) {
      loading.value = true;
      error.value = null;
      try {
        const imported = await invoke<AppConfig>('load_config_from_file', { path });
        targets.value = imported.targets;
        macros.value = imported.macros;
        hotkeys.value = imported.hotkeys;
        recordingsDirectory.value = imported.recordings_directory;
      } catch (e) {
        error.value = String(e);
        throw e;
      } finally {
        loading.value = false;
      }
    }

    // -----------------------------------------------------------------------
    // Target CRUD
    // -----------------------------------------------------------------------
    function addTarget(target: TargetSpec) {
      targets.value[target.name] = target;
    }

    function updateTarget(oldName: string, target: TargetSpec) {
      if (oldName !== target.name) {
        delete targets.value[oldName];
      }
      targets.value[target.name] = target;
    }

    function removeTarget(name: string) {
      delete targets.value[name];
    }

    // -----------------------------------------------------------------------
    // Macro CRUD
    // -----------------------------------------------------------------------
    function addMacro(macro: MacroSequence) {
      macros.value[macro.name] = macro;
    }

    function updateMacro(oldName: string, macro: MacroSequence) {
      if (oldName !== macro.name) {
        delete macros.value[oldName];
      }
      macros.value[macro.name] = macro;
    }

    function removeMacro(name: string) {
      delete macros.value[name];
    }

    // -----------------------------------------------------------------------
    // Window inspection
    // -----------------------------------------------------------------------
    async function refreshWindows() {
      loading.value = true;
      try {
        windows.value = await invoke<WindowMatch[]>('list_windows');
      } catch (e) {
        error.value = String(e);
      } finally {
        loading.value = false;
      }
    }

    async function findWindows(target: TargetSpec) {
      loading.value = true;
      try {
        return await invoke<WindowMatch[]>('find_windows', { target });
      } catch (e) {
        error.value = String(e);
        return [];
      } finally {
        loading.value = false;
      }
    }

    // -----------------------------------------------------------------------
    // Macro execution
    // -----------------------------------------------------------------------
    async function playMacro(name: string): Promise<boolean> {
      // Flush any pending sync before playing
      if (syncTimer) {
        clearTimeout(syncTimer);
        syncTimer = null;
        await doSync();
      }
      try {
        return await invoke<boolean>('play_macro', { macroName: name });
      } catch (e) {
        error.value = String(e);
        return false;
      }
    }

    async function playMacroDirect(sequence: MacroSequence, target: TargetSpec): Promise<boolean> {
      try {
        return await invoke<boolean>('play_macro_direct', { sequence, target });
      } catch (e) {
        error.value = String(e);
        return false;
      }
    }

    // -----------------------------------------------------------------------
    // Hotkey daemon control
    // -----------------------------------------------------------------------
    async function startDaemon(): Promise<boolean> {
      try {
        const result = await invoke<boolean>('start_hotkey_daemon');
        daemonRunning.value = result;
        return result;
      } catch (e) {
        error.value = String(e);
        return false;
      }
    }

    async function stopDaemon(): Promise<void> {
      try {
        await invoke('stop_hotkey_daemon');
        daemonRunning.value = false;
      } catch (e) {
        error.value = String(e);
      }
    }

    async function refreshDaemonStatus(): Promise<void> {
      try {
        daemonRunning.value = await invoke<boolean>('is_hotkey_daemon_running');
      } catch (e) {
        error.value = String(e);
      }
    }

    // -----------------------------------------------------------------------
    // Hotkey state event listener
    // -----------------------------------------------------------------------
    function setupHotkeyStateListener() {
      void listen<HotkeyStateEvent>('hotkey-state', (event) => {
        const { id, active, description, trigger_mode } = event.payload;
        hotkeyStates.value[id] = active;

        const label = description || `Hotkey #${id}`;
        if (trigger_mode === 'Once') {
          Notify.create({
            message: `${label} executed`,
            color: 'positive',
            position: 'bottom-right',
            timeout: 1500,
            icon: 'play_arrow',
          });
        } else if (active) {
          Notify.create({
            message: `${label} started`,
            color: 'positive',
            position: 'bottom-right',
            timeout: 2000,
            icon: 'play_arrow',
          });
        } else {
          Notify.create({
            message: `${label} stopped`,
            color: 'warning',
            position: 'bottom-right',
            timeout: 2000,
            icon: 'stop',
          });
        }
      });
    }

    // Start listening immediately
    setupHotkeyStateListener();

    return {
      // State
      targets,
      macros,
      hotkeys,
      recordingsDirectory,
      windows,
      loading,
      error,
      syncing,
      daemonRunning,
      hotkeyStates,
      // Computed
      targetCount,
      macroCount,
      hotkeyCount,
      config,
      // Actions
      importFromFile,
      addTarget,
      updateTarget,
      removeTarget,
      addMacro,
      updateMacro,
      removeMacro,
      refreshWindows,
      findWindows,
      playMacro,
      playMacroDirect,
      startDaemon,
      stopDaemon,
      refreshDaemonStatus,
    };
  },
  {
    persist: {
      pick: ['targets', 'macros', 'hotkeys', 'recordingsDirectory'],
    },
  },
);

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useAppStore, import.meta.hot));
}
