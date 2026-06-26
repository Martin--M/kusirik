<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings.store'
import IconLive from '../icons/IconLive.vue'
import IconMovies from '../icons/IconMovies.vue'
import IconSeries from '../icons/IconSeries.vue'
import IconSettings from '../icons/IconSettings.vue'
import IconChevron from '../icons/IconChevron.vue'
import IconSearch from '../icons/IconSearch.vue'

const route = useRoute()
const router = useRouter()
const settingsStore = useSettingsStore()

const navItems = [
  { name: 'live', label: 'Live TV', icon: IconLive },
  { name: 'movies', label: 'Movies', icon: IconMovies },
  { name: 'series', label: 'TV Series', icon: IconSeries },
  { name: 'settings', label: 'Settings', icon: IconSettings }
]

const activeRouteName = computed(() => route.name)
const isCollapsed = computed(() => settingsStore.sidebarCollapsed)
const searchQuery = ref('')

// Watch query param in URL to keep the input text synced (e.g. if the user uses browser back button)
watch(
  () => route.query.q,
  (newQ) => {
    searchQuery.value = (newQ as string) || ''
  },
  { immediate: true }
)

function handleSearchInput() {
  const val = searchQuery.value.trim()
  const targetQuery = val ? { q: val } : {}
  if (route.name !== 'search') {
    router.push({ name: 'search', query: targetQuery })
  } else {
    router.replace({ name: 'search', query: targetQuery })
  }
}

function focusSearchInput() {
  if (isCollapsed.value) {
    settingsStore.toggleSidebar()
  }
  router.push({ name: 'search', query: searchQuery.value ? { q: searchQuery.value } : {} })
}

function navigate(name: string) {
  router.push({ name })
}
</script>

<template>
  <aside class="app-sidebar" :class="{ collapsed: isCollapsed }">
    <div class="logo-container">
      <div class="logo-left">
        <div class="logo-icon"></div>
        <span class="logo-text">IPTV Helper</span>
      </div>
      <button
        class="collapse-toggle-btn"
        @click="settingsStore.toggleSidebar"
        :title="isCollapsed ? 'Expand Sidebar' : 'Collapse Sidebar'"
      >
        <IconChevron :direction="isCollapsed ? 'right' : 'left'" class="toggle-icon" />
      </button>
    </div>
    
    <div class="search-container" :class="{ collapsed: isCollapsed }">
      <button v-if="isCollapsed" class="search-icon-btn" @click="focusSearchInput" title="Search">
        <IconSearch class="search-icon" />
      </button>
      <div v-else class="search-input-wrapper">
        <IconSearch class="search-input-icon" />
        <input
          v-model="searchQuery"
          @input="handleSearchInput"
          type="text"
          placeholder="Search..."
          class="sidebar-search-input"
        />
      </div>
    </div>

    <nav class="nav-menu">
      <button
        v-for="item in navItems"
        :key="item.name"
        class="nav-item"
        :class="{ active: activeRouteName === item.name }"
        @click="navigate(item.name)"
        :title="isCollapsed ? item.label : undefined"
      >
        <component :is="item.icon" class="nav-icon" />
        <span class="nav-label">{{ item.label }}</span>
      </button>
    </nav>
  </aside>
</template>

<style scoped>
.app-sidebar {
  width: 240px;
  background-color: var(--color-surface);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  height: 100vh;
  position: sticky;
  top: 0;
  padding: var(--spacing-6) var(--spacing-4);
  flex-shrink: 0;
  transition: width var(--transition-normal) ease, padding var(--transition-normal) ease;
}

.app-sidebar.collapsed {
  width: 72px;
  padding: var(--spacing-6) var(--spacing-2);
}

.logo-container {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--spacing-8);
  padding: 0 var(--spacing-2);
  transition: all var(--transition-normal) ease;
}

.logo-left {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
}

.app-sidebar.collapsed .logo-container {
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-4);
  padding: 0;
}

.app-sidebar.collapsed .logo-left {
  justify-content: center;
}

.logo-icon {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  background: linear-gradient(135deg, var(--color-primary), #a855f7);
  box-shadow: 0 0 15px rgba(96, 165, 250, 0.3);
  flex-shrink: 0;
}

.logo-text {
  font-size: 1.25rem;
  font-weight: 800;
  letter-spacing: -0.025em;
  background: linear-gradient(135deg, #fff, var(--color-text-muted));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

[data-theme='light'] .logo-text {
  background: linear-gradient(135deg, var(--color-text), #555);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.nav-menu {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
}

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: var(--spacing-3) var(--spacing-4);
  border-radius: var(--radius-md);
  color: var(--color-text-muted);
  background: none;
  border: none;
  font-family: inherit;
  font-size: 0.95rem;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.app-sidebar.collapsed .nav-item {
  justify-content: center;
  padding: var(--spacing-3);
  gap: 0;
}

.nav-item:hover {
  color: var(--color-text);
  background-color: rgba(255, 255, 255, 0.03);
}

[data-theme='light'] .nav-item:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.nav-item.active {
  color: #fff;
  background-color: var(--color-primary);
  box-shadow: 0 4px 12px rgba(96, 165, 250, 0.2);
}

[data-theme='light'] .nav-item.active {
  color: #fff;
  background-color: var(--color-primary);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.2);
}

.nav-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.nav-label {
  font-weight: 600;
}

.collapse-toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  color: var(--color-text-muted);
  background: none;
  border: none;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
  flex-shrink: 0;
}

.collapse-toggle-btn:hover {
  color: var(--color-text);
  background-color: rgba(255, 255, 255, 0.05);
}

[data-theme='light'] .collapse-toggle-btn:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.toggle-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

.logo-text,
.nav-label {
  white-space: nowrap;
  opacity: 1;
  transition: opacity var(--transition-fast) ease, max-width var(--transition-fast) ease;
  max-width: 150px;
  overflow: hidden;
}

.app-sidebar.collapsed .logo-text,
.app-sidebar.collapsed .nav-label {
  opacity: 0;
  max-width: 0;
  pointer-events: none;
  margin: 0;
}

.search-container {
  margin-bottom: var(--spacing-6);
  padding: 0 var(--spacing-2);
  transition: all var(--transition-normal) ease;
}

.search-container.collapsed {
  display: flex;
  justify-content: center;
  padding: 0;
}

.search-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  color: var(--color-text-muted);
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--color-border);
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.search-icon-btn:hover {
  color: var(--color-text);
  background-color: rgba(255, 255, 255, 0.08);
}

.search-icon {
  width: 20px;
  height: 20px;
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
}

.search-input-icon {
  position: absolute;
  left: 12px;
  width: 16px;
  height: 16px;
  color: var(--color-text-muted);
  pointer-events: none;
}

.sidebar-search-input {
  width: 100%;
  height: 38px;
  padding: 0 var(--spacing-3) 0 36px;
  border-radius: var(--radius-md);
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--color-border);
  color: var(--color-text);
  font-family: inherit;
  font-size: 0.9rem;
  transition: all var(--transition-fast) ease;
}

.sidebar-search-input:focus {
  outline: none;
  border-color: var(--color-primary);
  background: rgba(255, 255, 255, 0.06);
}

[data-theme='light'] .sidebar-search-input {
  background: rgba(0, 0, 0, 0.02);
}

[data-theme='light'] .sidebar-search-input:focus {
  background: rgba(0, 0, 0, 0.04);
}
</style>
