<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

const navItems = [
  { name: 'live', label: 'Live TV', icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v10a1 1 0 01-1 1H5a1 1 0 01-1-1V5zm1 1v8h14V6H5z M7 18h10M12 16v2' },
  { name: 'movies', label: 'Movies', icon: 'M7 4v16M17 4v16M3 8h18M3 16h18 M9 12h6' },
  { name: 'series', label: 'Series', icon: 'M15 10l5 5-5 5M4 4h7a4 4 0 014 4v8a4 4 0 01-4 4H4z' },
  { name: 'settings', label: 'Settings', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z' }
]

const activeRouteName = computed(() => route.name)

function navigate(name: string) {
  router.push({ name })
}
</script>

<template>
  <nav class="app-bottom-nav">
    <button
      v-for="item in navItems"
      :key="item.name"
      class="nav-tab"
      :class="{ active: activeRouteName === item.name }"
      @click="navigate(item.name)"
    >
      <svg class="tab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path :d="item.icon" />
      </svg>
      <span class="tab-label">{{ item.label }}</span>
    </button>
  </nav>
</template>

<style scoped>
.app-bottom-nav {
  height: 64px;
  background-color: var(--color-surface);
  border-top: 1px solid var(--color-border);
  display: flex;
  justify-content: space-around;
  align-items: center;
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 100;
  padding-bottom: env(safe-area-inset-bottom);
}

.nav-tab {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-1);
  background: none;
  border: none;
  height: 100%;
  color: var(--color-text-muted);
  font-family: inherit;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.nav-tab:hover {
  color: var(--color-text);
}

.nav-tab.active {
  color: var(--color-primary);
}

.tab-icon {
  width: 22px;
  height: 22px;
}

.tab-label {
  font-size: 0.75rem;
  font-weight: 600;
}
</style>
