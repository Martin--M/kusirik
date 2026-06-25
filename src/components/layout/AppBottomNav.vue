<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import IconLive from '../icons/IconLive.vue'
import IconMovies from '../icons/IconMovies.vue'
import IconSeries from '../icons/IconSeries.vue'
import IconSettings from '../icons/IconSettings.vue'

const route = useRoute()
const router = useRouter()

const navItems = [
  { name: 'live', label: 'Live TV', icon: IconLive },
  { name: 'movies', label: 'Movies', icon: IconMovies },
  { name: 'series', label: 'Series', icon: IconSeries },
  { name: 'settings', label: 'Settings', icon: IconSettings }
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
      <component :is="item.icon" class="tab-icon" />
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
