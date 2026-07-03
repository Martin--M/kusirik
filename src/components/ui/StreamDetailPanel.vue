<script setup lang="ts">
import { ref } from 'vue'
import CachedImage from './CachedImage.vue'
import IconPlay from '../icons/IconPlay.vue'
import IconCopy from '../icons/IconCopy.vue'

const props = withDefaults(
  defineProps<{
    stream: {
      stream_id: number
      name?: string | null
      stream_icon?: string | null
      [key: string]: any
    } | null
    isMobileOpen: boolean
    playButtonText?: string
    copyButtonText?: string
    showActions?: boolean
    squareImage?: boolean
    defaultWidth?: number
    playDisabled?: boolean
  }>(),
  {
    showActions: true,
    squareImage: false,
    defaultWidth: 320,
    playDisabled: false
  }
)

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'play'): void
  (e: 'copy'): void
}>()

const sidebarWidth = ref(props.defaultWidth)
const isResizing = ref(false)

function startResize(e: MouseEvent) {
  isResizing.value = true
  const startX = e.clientX
  const startWidth = sidebarWidth.value

  function doResize(moveEvent: MouseEvent) {
    const delta = moveEvent.clientX - startX
    const newWidth = startWidth - delta
    // Enforce limits: min 280px, max 1000px
    sidebarWidth.value = Math.max(280, Math.min(1000, newWidth))
  }

  function stopResize() {
    isResizing.value = false
    window.removeEventListener('mousemove', doResize)
    window.removeEventListener('mouseup', stopResize)
  }

  window.addEventListener('mousemove', doResize)
  window.addEventListener('mouseup', stopResize)
}
</script>

<template>
  <!-- Desktop Sidebar Panel -->
  <!-- Desktop Sidebar Panel -->
  <transition name="slide-fade">
    <div
      v-if="stream"
      class="details-sidebar-wrapper desktop-only"
      :style="{ width: `${sidebarWidth}px` }"
    >
      <aside class="details-sidebar">
        <div class="details-panel" :style="{ width: `${sidebarWidth}px` }">
          <button class="close-details-btn" @click="emit('close')" title="Close Details">×</button>
          
          <div class="details-header">
            <div class="details-poster" :class="{ 'aspect-square': squareImage }">
              <CachedImage
                :src="stream.stream_icon"
                :alt="stream.name || 'Stream Image'"
                :fallback-text="stream.name || ''"
              />
            </div>
            <h3 class="stream-name-title">{{ stream.name || 'Unnamed Stream' }}</h3>
            <!-- Slot for subtitle, badges or extra header content -->
            <slot name="header-meta"></slot>
          </div>

          <!-- Scrollable details list -->
          <div class="details-scrollable-body">
            <slot></slot>
          </div>

          <div v-if="showActions" class="action-buttons">
            <button class="btn btn-primary" :disabled="playDisabled" @click="emit('play')">
              <slot name="play-icon">
                <IconPlay class="btn-icon" />
              </slot>
              {{ playButtonText || 'Play' }}
            </button>
            <button class="btn btn-secondary" @click="emit('copy')">
              <slot name="copy-icon">
                <IconCopy class="btn-icon" />
              </slot>
              {{ copyButtonText || 'Copy URL' }}
            </button>
          </div>
        </div>
      </aside>
      <!-- Resize Handle -->
      <div class="resize-handle" @mousedown.prevent="startResize"></div>
    </div>
  </transition>

  <!-- Mobile Bottom Sheet Details Panel -->
  <transition name="slide-up">
    <div v-if="stream && isMobileOpen" class="bottom-sheet-backdrop mobile-only" @click="emit('close')">
      <div class="bottom-sheet-content" @click.stop>
        <div class="drag-handle"></div>
        <button class="close-sheet" @click="emit('close')">×</button>
        
        <div class="details-header mobile">
          <div class="details-poster mobile" :class="{ 'aspect-square': squareImage }">
            <CachedImage
              :src="stream.stream_icon"
              :alt="stream.name || 'Stream Image'"
              :fallback-text="stream.name || ''"
            />
          </div>
          <div class="header-text">
            <h3 class="stream-name-title">{{ stream.name || 'Unnamed Stream' }}</h3>
            <slot name="header-meta-mobile"></slot>
          </div>
        </div>

        <div class="details-scrollable-body mobile">
          <slot name="mobile-body">
            <slot></slot>
          </slot>
        </div>

        <div v-if="showActions" class="action-buttons mobile">
          <button class="btn btn-primary" :disabled="playDisabled" @click="emit('play')">
            {{ playButtonText || 'Play' }}
          </button>
          <button class="btn btn-secondary" @click="emit('copy')">
            {{ copyButtonText || 'Copy URL' }}
          </button>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
/* Details Sidebar (Desktop) */
.details-sidebar-wrapper {
  position: relative;
  height: 100%;
  flex-shrink: 0;
  display: flex;
  transition: width var(--transition-normal) ease, opacity var(--transition-normal) ease, transform var(--transition-normal) ease;
}

.details-sidebar {
  width: 100%;
  background-color: rgba(30, 41, 59, 0.4);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  height: 100%;
  box-sizing: border-box;
}

.resize-handle {
  position: absolute;
  top: 0;
  left: -3px;
  width: 6px;
  height: 100%;
  cursor: col-resize;
  background-color: transparent;
  transition: background-color var(--transition-fast) ease;
  z-index: 100;
}

.resize-handle:hover,
.details-sidebar-wrapper:active .resize-handle {
  background-color: var(--color-primary);
}

[data-theme='light'] .details-sidebar {
  background-color: rgba(240, 240, 240, 0.4);
}

.details-panel {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-6);
  gap: var(--spacing-6);
  position: relative;
  box-sizing: border-box;
}

.close-details-btn {
  position: absolute;
  top: 16px;
  right: 16px;
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 1.5rem;
  cursor: pointer;
  line-height: 1;
  padding: 0;
  transition: color var(--transition-fast);
  z-index: 10;
}

.close-details-btn:hover {
  color: var(--color-text);
}

.details-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: var(--spacing-3);
}

.details-poster {
  width: 140px;
  aspect-ratio: 2/3;
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-lg), 0 0 35px rgba(96, 165, 250, 0.15);
}

.details-poster.aspect-square {
  aspect-ratio: 1/1;
}

.stream-name-title {
  font-size: 1.1rem;
  color: var(--color-text);
  font-weight: 700;
  margin: 0;
}

.details-scrollable-body {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.action-buttons {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-2);
  padding: var(--spacing-3) var(--spacing-4);
  font-family: inherit;
  font-size: 0.95rem;
  font-weight: 600;
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  transition: all var(--transition-fast);
  width: 100%;
}

.btn-primary {
  background-color: var(--color-primary);
  color: #fff;
  box-shadow: 0 4px 12px rgba(96, 165, 250, 0.2);
}

.btn-primary:hover {
  background-color: var(--color-primary-hover);
  transform: translateY(-1px);
}

.btn-secondary {
  background-color: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--color-border);
  color: var(--color-text);
}

.btn-secondary:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.btn-icon {
  width: 18px;
  height: 18px;
}

/* Transitions */
.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: width var(--transition-normal) ease, opacity var(--transition-normal) ease, transform var(--transition-normal) ease;
  overflow: hidden;
}
.slide-fade-enter-active .details-panel,
.slide-fade-leave-active .details-panel {
  width: 320px !important;
}
.slide-fade-enter-from,
.slide-fade-leave-to {
  width: 0px !important;
  opacity: 0;
  transform: translateX(40px);
}

/* Mobile Bottom Sheet Details Panel styles */
.bottom-sheet-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  z-index: 1000;
}

.bottom-sheet-content {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  max-height: 80vh;
  background-color: rgba(15, 23, 42, 0.95);
  backdrop-filter: blur(20px);
  border-top: 1px solid var(--color-border);
  border-top-left-radius: var(--radius-lg);
  border-top-right-radius: var(--radius-lg);
  padding: var(--spacing-6);
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
  box-shadow: 0 -10px 25px rgba(0,0,0,0.5);
  overflow-y: auto;
}

[data-theme='light'] .bottom-sheet-content {
  background-color: rgba(255, 255, 255, 0.95);
  box-shadow: 0 -10px 25px rgba(0,0,0,0.1);
}

.drag-handle {
  width: 40px;
  height: 4px;
  background-color: var(--color-border);
  border-radius: 9999px;
  margin: 0 auto;
}

.close-sheet {
  position: absolute;
  top: 16px;
  right: 16px;
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 1.5rem;
  cursor: pointer;
  line-height: 1;
}

.details-header.mobile {
  display: flex;
  align-items: center;
  text-align: left;
  gap: var(--spacing-4);
}

.details-poster.mobile {
  width: 70px;
  aspect-ratio: 2/3;
  border-radius: var(--radius-md);
  overflow: hidden;
  flex-shrink: 0;
}

.header-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.details-scrollable-body.mobile {
  overflow-y: auto;
  max-height: 40vh;
}

.action-buttons.mobile {
  display: flex;
  gap: var(--spacing-3);
  margin-top: var(--spacing-2);
}

/* Animations for slide up drawer */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: opacity 0.3s ease, transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
}

.slide-up-enter-from .bottom-sheet-content {
  transform: translateY(100%);
}

.slide-up-leave-to .bottom-sheet-content {
  transform: translateY(100%);
}

/* Responsive displays helper elements */
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
</style>
