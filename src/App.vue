<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useSettingsStore } from '@/stores/settings.store'
import { useSync } from '@/composables/useSync'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import AppBottomNav from '@/components/layout/AppBottomNav.vue'
import AppTopBar from '@/components/layout/AppTopBar.vue'
import AppTitleBar from '@/components/layout/AppTitleBar.vue'
import AppToast from '@/components/ui/AppToast.vue'
import { checkIsDesktop } from '@/lib/device'

const settingsStore = useSettingsStore()
const route = useRoute()
useSync() // Initialize sync listeners

onMounted(async () => {
  await settingsStore.load()
  settingsStore.applyTheme()
})

const showLayout = computed(() => {
  return route.meta.requiresProfile === true || route.name === 'settings'
})

const isDesktop = computed(() => {
  return checkIsDesktop()
})
</script>

<template>
  <div class="app-window-wrapper">
    <!-- Custom Window Title Bar (Desktop Only) -->
    <AppTitleBar v-if="isDesktop" />

    <div class="app-layout">
      <!-- Desktop Sidebar -->
      <AppSidebar v-if="showLayout" class="desktop-only" />

      <div class="main-container">
        <!-- Top header bar -->
        <AppTopBar v-if="showLayout && route.name !== 'guide'" />

        <!-- Main screen viewport with Keep-Alive view caching -->
        <main class="content-area">
          <router-view v-slot="{ Component }">
            <keep-alive>
              <component :is="Component" />
            </keep-alive>
          </router-view>
        </main>

        <!-- Mobile Bottom navigation -->
        <AppBottomNav v-if="showLayout" class="mobile-only" />
      </div>
    </div>

    <!-- Global Dynamic Toast Notification System -->
    <AppToast />
  </div>
</template>

<style>
/* Global CSS helpers for responsive displays */
@media (min-width: 769px) {
  .mobile-only {
    display: none !important;
  }
}
@media (max-width: 768px) {
  .desktop-only {
    display: none !important;
  }
}

.app-window-wrapper {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: var(--color-bg);
  color: var(--color-text);
  overflow: hidden;
}

.app-layout {
  flex: 1;
  display: flex;
  min-height: 0;
  overflow: hidden;
}

.main-container {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  height: 100%;
  position: relative;
}

.content-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

/* Mobile specific styling adjustments */
@media (max-width: 768px) {
  .main-container {
    padding-bottom: 64px; /* Space for bottom nav */
  }
}
</style>
