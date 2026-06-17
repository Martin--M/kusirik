<script setup lang="ts">
import { onMounted } from 'vue'
import { useSettingsStore } from '@/stores/settings.store'
import { useSync } from '@/composables/useSync'

const settingsStore = useSettingsStore()
useSync() // Initialize sync listeners

onMounted(async () => {
  await settingsStore.load()
  settingsStore.applyTheme()
})
</script>

<template>
  <div class="app-layout">
    <router-view />
  </div>
</template>

<style>
.app-layout {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
</style>
