<script lang="ts" setup>
import { storeToRefs } from 'pinia';

import { i18nSubPath } from 'src/utils/common';
import { useSettingsStore } from 'stores/settings';
import { computed } from 'vue';

const { toggleTheme } = useSettingsStore();
const { darkMode } = storeToRefs(useSettingsStore());

const i18n = i18nSubPath('components.ThemeButton');

const buttonProps = computed(() => {
  switch (darkMode.value) {
    case false:
      return { color: 'orange', icon: 'light_mode' };
    case 'auto':
      return { color: 'teal', icon: 'hdr_auto' };
    default:
      return { color: 'yellow', icon: 'dark_mode' };
  }
});
</script>

<template>
  <q-btn :icon="buttonProps.icon" :text-color="buttonProps.color" flat round @click="toggleTheme">
    <q-tooltip
      class="text-no-wrap"
      anchor="center right"
      self="center left"
      transition-hide="jump-left"
      transition-show="jump-right"
    >
      {{ i18n('labels.switchTheme') }}
    </q-tooltip>
  </q-btn>
</template>

<style scoped></style>
