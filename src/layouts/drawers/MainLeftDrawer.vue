<script setup lang="ts">
import { bus } from 'boot/bus';
import { MAIN_NAVIGATIONS } from 'components/navigations';
</script>

<template>
  <q-drawer
    behavior="desktop"
    bordered
    no-swipe-backdrop
    no-swipe-close
    no-swipe-open
    side="left"
    show-if-above
    @mini-state="bus.emit('drawer', $event ? 'minimize' : 'maximize', 'left')"
    @show="bus.emit('drawer', 'open', 'left')"
    @hide="bus.emit('drawer', 'close', 'left')"
  >
    <q-list separator>
      <template v-for="navigation in MAIN_NAVIGATIONS" :key="navigation.route">
        <q-item :disable="!navigation.available" exact :to="navigation.route">
          <q-item-section avatar>
            <q-icon :name="navigation.icon" :color="navigation.available ? '' : 'grey'" />
          </q-item-section>
          <q-item-section :class="navigation.available ? '' : 'text-grey'">
            {{ navigation.label }}
          </q-item-section>
        </q-item>
      </template>
    </q-list>
  </q-drawer>
</template>

<style scoped></style>
