<script setup lang="ts">
import ThemeButton from 'components/ThemeButton.vue';

import { bus } from 'boot/bus';
import { i18nSubPath } from 'src/utils/common';
import LocaleButton from 'components/LocaleButton.vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

const i18n = i18nSubPath('layouts.headers.MainHeader');

const appWindow = getCurrentWindow();
const minimize = () => appWindow.minimize();
const toggleMaximize = () => appWindow.toggleMaximize();
const closeApp = () => appWindow.close();
</script>

<template>
  <q-header bordered class="bg-primary text-white">
    <q-bar class="q-electron-drag">
      <q-icon name="map" class="q-electron-drag--exception" />
      <div>
        {{ i18n('labels.title') }}
      </div>
      <q-space />
      <q-btn dense flat icon="minimize" @click="minimize" />
      <q-btn dense flat icon="crop_square" @click="toggleMaximize" />
      <q-btn dense flat icon="close" @click="closeApp" />
    </q-bar>
    <q-toolbar>
      <q-btn flat icon="menu" round @click="bus.emit('drawer', 'toggle', 'left')" />
      <q-toolbar-title shrink>
        <q-avatar>
          <q-img src="favicon.ico" />
          <!--          <q-img src="~assets/logos/light/simple.svg" />-->
        </q-avatar>
        {{ i18n('labels.title') }}
      </q-toolbar-title>
      <q-space />
      <div class="row q-gutter-x-sm">
        <theme-button />
        <locale-button />
        <q-btn flat icon="menu" round @click="bus.emit('drawer', 'toggle', 'right')" />
      </div>
    </q-toolbar>
  </q-header>
</template>

<style scoped></style>
