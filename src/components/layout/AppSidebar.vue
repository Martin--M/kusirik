<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings.store'

const route = useRoute()
const router = useRouter()
const settingsStore = useSettingsStore()

const navItems = [
  { name: 'live', label: 'Live TV', icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v10a1 1 0 01-1 1H5a1 1 0 01-1-1V5zm1 1v8h14V6H5z M7 18h10M12 16v2' },
  { name: 'movies', label: 'Movies', icon: 'M7 4v16M17 4v16M3 8h18M3 16h18 M9 12h6' },
  { name: 'series', label: 'TV Series', icon: 'M15 10l5 5-5 5M4 4h7a4 4 0 014 4v8a4 4 0 01-4 4H4z' },
  { name: 'settings', label: 'Settings', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z' }
]

const activeRouteName = computed(() => route.name)
const isCollapsed = computed(() => settingsStore.sidebarCollapsed)

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
        <svg class="toggle-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path v-if="isCollapsed" d="M9 18l6-6-6-6" />
          <path v-else d="M15 18l-6-6 6-6" />
        </svg>
      </button>
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
        <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path :d="item.icon" />
        </svg>
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
</style>
