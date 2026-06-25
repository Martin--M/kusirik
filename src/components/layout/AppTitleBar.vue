<script setup lang="ts">
import { ref, onMounted } from 'vue'
import IconClose from '../icons/IconClose.vue'

const isMaximized = ref(false)
let appWindow: any = null

onMounted(async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    appWindow = getCurrentWindow()
    isMaximized.value = await appWindow.isMaximized()
    
    // Listen to resize events to keep maximized state in sync
    await appWindow.onResized(async () => {
      isMaximized.value = await appWindow.isMaximized()
    })
  } catch (e) {
    console.warn('Tauri window APIs not available (likely running in a web browser):', e)
  }
})

async function minimize() {
  if (appWindow) {
    try {
      await appWindow.minimize()
    } catch (e) {
      console.error('Failed to minimize window:', e)
    }
  }
}

async function toggleMaximize() {
  if (appWindow) {
    try {
      await appWindow.toggleMaximize()
      isMaximized.value = await appWindow.isMaximized()
    } catch (e) {
      console.error('Failed to toggle maximize window:', e)
    }
  }
}

async function closeApp() {
  if (appWindow) {
    try {
      await appWindow.close()
    } catch (e) {
      console.error('Failed to close window:', e)
    }
  }
}

// Programmatic window dragging fallback
function handleMouseDown(e: MouseEvent) {
  // Only start drag on left click and if not clicking a window control button
  if (e.button === 0 && !(e.target as HTMLElement).closest('.control-btn')) {
    if (appWindow) {
      try {
        appWindow.startDragging()
      } catch (err) {
        console.error('Failed to start window dragging:', err)
      }
    }
  }
}

// Programmatic double click to maximize
function handleDblClick(e: MouseEvent) {
  if (!(e.target as HTMLElement).closest('.control-btn')) {
    toggleMaximize()
  }
}
</script>

<template>
  <div 
    class="app-title-bar" 
    data-tauri-drag-region
    @mousedown="handleMouseDown"
    @dblclick="handleDblClick"
  >
    <!-- Left Section: Logo & Name -->
    <div class="logo-section" data-tauri-drag-region>
      <div class="app-logo" data-tauri-drag-region></div>
      <span class="app-name" data-tauri-drag-region>IPTV Helper</span>
    </div>

    <!-- Center Spacer (Drag Region) -->
    <div class="drag-spacer" data-tauri-drag-region></div>

    <!-- Right Section: Window Controls -->
    <div class="window-controls">
      <button class="control-btn minimize-btn" @click="minimize" title="Minimize">
        <svg viewBox="0 0 10 1" class="control-icon-svg">
          <line x1="0" y1="0.5" x2="10" y2="0.5" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>

      <button class="control-btn maximize-btn" @click="toggleMaximize" :title="isMaximized ? 'Restore' : 'Maximize'">
        <!-- Maximize square icon -->
        <svg v-if="!isMaximized" viewBox="0 0 10 10" class="control-icon-svg">
          <rect x="1" y="1" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1" />
        </svg>
        <!-- Restore double squares icon -->
        <svg v-else viewBox="0 0 10 10" class="control-icon-svg">
          <rect x="3" y="1" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1" />
          <rect x="1" y="3" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>

      <button class="control-btn close-btn" @click="closeApp" title="Close">
        <IconClose class="close-icon" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.app-title-bar {
  height: 32px;
  background-color: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  user-select: none;
  font-family: inherit;
  font-size: 0.75rem;
  font-weight: 600;
  z-index: 9999;
  flex-shrink: 0;
}

[data-theme='light'] .app-title-bar {
  background-color: var(--color-surface);
}

.logo-section {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding-left: var(--spacing-4);
  color: var(--color-text-muted);
}

.app-logo {
  width: 12px;
  height: 12px;
  background-color: var(--color-primary);
  border-radius: 3px;
}

.app-name {
  color: var(--color-text-muted);
}

.drag-spacer {
  flex: 1;
  height: 100%;
}

.window-controls {
  display: flex;
  height: 100%;
}

.control-btn {
  width: 46px;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: default;
  transition: background-color var(--transition-fast), color var(--transition-fast);
  outline: none;
}

.control-btn:hover {
  background-color: rgba(255, 255, 255, 0.05);
  color: var(--color-text);
}

[data-theme='light'] .control-btn:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.close-btn:hover {
  background-color: #ef4444 !important;
  color: #ffffff !important;
}

.control-icon-svg {
  width: 10px;
  height: 10px;
}

.close-icon {
  width: 10px;
  height: 10px;
}
</style>
