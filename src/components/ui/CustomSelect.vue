<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import IconChevron from '../icons/IconChevron.vue'

defineProps<{
  modelValue: string
  options: Record<string, string>
  ariaLabel?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const isOpen = ref(false)

function selectOption(value: string) {
  emit('update:modelValue', value)
  isOpen.value = false
}

function closeDropdown(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.custom-dropdown')) {
    isOpen.value = false
  }
}

onMounted(() => {
  window.addEventListener('click', closeDropdown)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdown)
})
</script>

<template>
  <div class="custom-dropdown">
    <button
      class="dropdown-trigger"
      @click.stop="isOpen = !isOpen"
      aria-haspopup="listbox"
      :aria-expanded="isOpen"
      :title="ariaLabel || 'Select Option'"
    >
      <span>{{ options[modelValue] }}</span>
      <IconChevron direction="down" class="dropdown-chevron" />
    </button>
    <transition name="dropdown-fade">
      <ul v-if="isOpen" class="dropdown-menu">
        <li
          v-for="(label, value) in options"
          :key="value"
          class="dropdown-item"
          :class="{ active: modelValue === value }"
          @click="selectOption(value as string)"
        >
          {{ label }}
        </li>
      </ul>
    </transition>
  </div>
</template>

<style scoped>
.custom-dropdown {
  position: relative;
  display: inline-block;
}

.dropdown-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-2);
  padding: var(--spacing-2) var(--spacing-3);
  min-width: 120px;
  height: 34px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background-color: rgba(15, 23, 42, 0.4);
  color: var(--color-text);
  font-family: inherit;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  outline: none;
  transition: all var(--transition-fast);
}

[data-theme='light'] .dropdown-trigger {
  background-color: rgba(255, 255, 255, 0.6);
}

.dropdown-trigger:hover {
  border-color: var(--color-primary);
  background-color: rgba(255, 255, 255, 0.05);
}

[data-theme='light'] .dropdown-trigger:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.dropdown-chevron {
  width: 14px;
  height: 14px;
  color: var(--color-text-muted);
}

.dropdown-menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  min-width: 100%;
  background-color: rgba(15, 23, 42, 0.95);
  backdrop-filter: blur(10px);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-1) 0;
  margin: 0;
  list-style: none;
  z-index: 50;
  box-shadow: var(--shadow-lg), 0 10px 30px rgba(0, 0, 0, 0.5);
  transform-origin: top;
}

[data-theme='light'] .dropdown-menu {
  background-color: rgba(255, 255, 255, 0.95);
  box-shadow: var(--shadow-lg), 0 10px 30px rgba(0, 0, 0, 0.15);
}

.dropdown-item {
  padding: var(--spacing-2) var(--spacing-4);
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.dropdown-item:hover {
  color: var(--color-text);
  background-color: rgba(255, 255, 255, 0.05);
}

[data-theme='light'] .dropdown-item:hover {
  background-color: rgba(0, 0, 0, 0.03);
}

.dropdown-item.active {
  color: var(--color-primary);
  background-color: rgba(96, 165, 250, 0.08);
  font-weight: 700;
}

[data-theme='light'] .dropdown-item.active {
  background-color: rgba(59, 130, 246, 0.05);
}

.dropdown-fade-enter-active,
.dropdown-fade-leave-active {
  transition: opacity var(--transition-fast) ease, transform var(--transition-fast) ease;
}

.dropdown-fade-enter-from,
.dropdown-fade-leave-to {
  opacity: 0;
  transform: translateY(-8px) scaleY(0.95);
}
</style>
