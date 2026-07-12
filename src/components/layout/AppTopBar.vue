<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useSyncStore } from '@/stores/sync.store'
import { useI18n } from '@/composables/useI18n'
import { useRouter } from 'vue-router'
import { checkIsAndroid } from '@/lib/device'
import IconSearch from '../icons/IconSearch.vue'

const route = useRoute()
const router = useRouter()
const syncStore = useSyncStore()
const { t } = useI18n()

const showSearchIcon = computed(() => {
  return checkIsAndroid() && route.name !== 'search'
})

function goToSearch() {
  router.push({ name: 'search' })
}

const title = computed(() => {
  switch (route.name) {
    case 'live':
      return t('sidebar.live')
    case 'movies':
      return t('sidebar.movies')
    case 'series':
      return t('sidebar.series')
    case 'settings':
      return t('sidebar.settings')
    case 'search':
      return t('sidebar.search')
    case 'favorites':
      return t('sidebar.favorites')
    case 'history':
      return t('sidebar.history')
    default:
      return 'Kusirik'
  }
})

const isAnySyncing = computed(() => {
  return Object.values(syncStore.statuses).some((profileStatuses) =>
    Object.values(profileStatuses).some((s) => s.is_syncing)
  )
})

const currentSyncStatus = computed(() => {
  for (const profileStatuses of Object.values(syncStore.statuses)) {
    const active = Object.values(profileStatuses).find((s) => s.is_syncing)
    if (active) {
      const typeKey = `settings.stats.types.${active.data_type}`
      const name = t(typeKey)
      return t('setup.syncScreen.syncing') + ` (${name})...`
    }
  }
  return ''
})
</script>


<template>
  <header class="app-top-bar">
    <div class="title-section">
      <h2 class="view-title">{{ title }}</h2>
    </div>

    <div class="status-section">
      <button
        v-if="showSearchIcon"
        class="search-icon-btn mobile-only"
        @click="goToSearch"
        :title="$t('sidebar.search')"
      >
        <IconSearch class="search-icon" />
      </button>

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

.search-icon-btn {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.search-icon-btn:hover {
  color: var(--color-text);
  background-color: var(--color-surface-hover);
}

.search-icon {
  width: 20px;
  height: 20px;
}
</style>
