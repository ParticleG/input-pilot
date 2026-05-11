<script setup lang="ts">
import { onMounted } from 'vue';
import { useAppStore } from 'stores/app';

const app = useAppStore();

onMounted(() => {
  void app.refreshWindows();
});
</script>

<template>
  <q-page class="q-pa-md">
    <div class="row items-center q-mb-md">
      <div class="text-h5">Windows</div>
      <q-space />
      <q-btn flat icon="refresh" label="Refresh" :loading="app.loading" @click="app.refreshWindows()" />
    </div>
    <q-table
      :rows="app.windows"
      :columns="[
        { name: 'handle', label: 'Handle', field: 'handle', align: 'left', format: (v: number) => '0x' + v.toString(16).toUpperCase() },
        { name: 'process', label: 'Process', field: 'process_name', align: 'left', sortable: true },
        { name: 'class', label: 'Class', field: 'class_name', align: 'left' },
        { name: 'title', label: 'Title', field: 'title', align: 'left', sortable: true },
      ]"
      row-key="handle"
      flat
      bordered
      :loading="app.loading"
      :pagination="{ rowsPerPage: 50 }"
    />
  </q-page>
</template>
