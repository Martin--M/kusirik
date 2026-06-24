<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useSyncStore } from '@/stores/sync.store'

const route = useRoute()
const syncStore = useSyncStore()

const title = computed(() => {
  switch (route.name) {
    case 'live':
      return 'Live TV'
    case 'movies':
      return 'Movies'
    case 'series':
      return 'TV Series'
    case 'settings':
      return 'Settings'
    case 'search':
      return 'Search'
    default:
      return 'IPTV Helper'
  }
})

const isAnySyncing = computed(() => {
  return Object.values(syncStore.statuses).some((s) => s.is_syncing)
})

const currentSyncStatus = computed(() => {
  const active = Object.values(syncStore.statuses).find((s) => s.is_syncing)
  if (!active) return ''
  const label = active.data_type.replace('_', ' ')
  return `Syncing ${label}...`
})
</script>

<template>
  <header class="app-top-bar">
    <div class="title-section">
      <h2 class="view-title">{{ title }}</h2>
    </div>

    <div class="status-section">
      <div v-if="isAnySyncing" class="sync-indicator">
        <span class="spinner-icon"></span>
        <span class="sync-text desktop-only">{{ currentSyncStatus }}</span>
      </div>
    </div>
  </header>
</template>

<style scoped>
.app-top-bar {
  height: 64px;
  background-color: rgba(15, 23, 42, 0.4);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 var(--spacing-6);
  position: sticky;
  top: 0;
  z-index: 50;
}

[data-theme='light'] .app-top-bar {
  background-color: rgba(255, 255, 255, 0.4);
}

.view-title {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--color-text);
}

.status-section {
  display: flex;
  align-items: center;
  gap: var(--spacing-4);
}

.sync-indicator {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  background: rgba(96, 165, 250, 0.1);
  border: 1px solid rgba(96, 165, 250, 0.2);
  padding: var(--spacing-1) var(--spacing-3);
  border-radius: 9999px;
  color: var(--color-primary);
  font-size: 0.85rem;
  font-weight: 600;
}

.spinner-icon {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(96, 165, 250, 0.3);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.sync-text {
  text-transform: capitalize;
}
</style>
