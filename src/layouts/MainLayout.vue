<script setup lang="ts">
import { useQuasar } from 'quasar';
import { ref } from 'vue';

import { bus } from 'boot/bus';

const { screen } = useQuasar();

const leftDrawerMini = ref(false);
const leftDrawerOpen = ref(false);
const rightDrawerMini = ref(false);
const rightDrawerOpen = ref(false);

bus.on('drawer', (action, position) => {
  const targetDrawerMini = position === 'left' ? leftDrawerMini : rightDrawerMini;
  const targetDrawerOpen = position === 'left' ? leftDrawerOpen : rightDrawerOpen;
  switch (action) {
    case 'open':
      targetDrawerOpen.value = true;
      break;
    case 'close':
      targetDrawerOpen.value = false;
      break;
    case 'toggle':
      targetDrawerOpen.value = !targetDrawerOpen.value;
      break;
    case 'maximize':
      targetDrawerMini.value = false;
      break;
    case 'minimize':
      targetDrawerMini.value = true;
      break;
    case 'switch':
      targetDrawerMini.value = !targetDrawerMini.value;
      break;
  }
});
</script>

<template>
  <q-layout view="hHh LpR lFf">
    <router-view :mobile="screen.lt.md" name="header" />
    <router-view
      :mobile="screen.lt.md"
      :model-value="leftDrawerOpen"
      :mini="leftDrawerMini"
      name="leftDrawer"
    />
    <q-page-container style="height: 100vh">
      <router-view />
    </q-page-container>
    <router-view
      :mobile="screen.lt.md"
      :model-value="rightDrawerOpen"
      :mini="rightDrawerMini"
      name="rightDrawer"
    />
    <router-view :mobile="screen.lt.md" name="footer" />
  </q-layout>
</template>
