import { defineStore, acceptHMRUpdate } from 'pinia';
import { Dark } from 'quasar';
import type { Ref } from 'vue';
import { ref } from 'vue';

import { i18nGlobal } from 'boot/i18n';
import { DARK_MODES } from 'stores/settings/constants';

export const useSettingsStore = defineStore(
  'settings',
  () => {
    const darkMode = ref<Dark['mode']>(Dark.mode);
    const locale = ref((i18nGlobal.locale as unknown as Ref<string>).value);

    const applyTheme = () => {
      Dark.set(darkMode.value);
    };

    const toggleTheme = () => {
      const index = DARK_MODES.indexOf(darkMode.value);
      darkMode.value = DARK_MODES[(index + 1) % DARK_MODES.length]!;
      applyTheme();
    };

    const updateLocale = (newLocale?: string) => {
      if (newLocale) {
        locale.value = newLocale;
      }
      (i18nGlobal.locale as unknown as Ref<string>).value = locale.value;
    };

    return {
      darkMode,
      locale,
      applyTheme,
      toggleTheme,
      updateLocale,
    };
  },
  {
    persist: true,
  },
);

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useSettingsStore, import.meta.hot));
}
