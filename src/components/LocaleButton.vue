<script setup lang="ts">
import { storeToRefs } from 'pinia';
import allLanguages from 'quasar/lang/index.json';

import messages from 'src/i18n';
import { useSettingsStore } from 'stores/settings';

const { updateLocale } = useSettingsStore();
const { locale } = storeToRefs(useSettingsStore());

const locales: typeof allLanguages = allLanguages.filter((lang) =>
  Object.keys(messages).includes(lang.isoName),
);

const setLocale = (newLocale: string) => {
  updateLocale(newLocale);
  window.location.reload();
};
</script>

<template>
  <q-btn flat icon="translate" round>
    <q-menu>
      <q-list>
        <q-item
          v-for="(item, index) in locales"
          :key="index"
          clickable
          @click="setLocale(item.isoName)"
        >
          <q-item-section>
            <q-item-label>
              {{ item.nativeName }}
            </q-item-label>
          </q-item-section>
          <q-item-section side>
            <q-icon v-if="locale === item.isoName" name="check" />
          </q-item-section>
        </q-item>
      </q-list>
    </q-menu>
  </q-btn>
</template>

<style scoped></style>
