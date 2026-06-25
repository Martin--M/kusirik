<script setup lang="ts">
import CustomSelect from './CustomSelect.vue'
import IconSearch from '../icons/IconSearch.vue'
import IconList from '../icons/IconList.vue'
import IconGrid from '../icons/IconGrid.vue'
import IconSort from '../icons/IconSort.vue'

const props = defineProps<{
  searchQuery: string
  searchPlaceholder?: string
  sortField?: string
  sortLabels?: Record<string, string>
  sortOrder: 'asc' | 'desc'
  isGridView?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:searchQuery', val: string): void
  (e: 'update:sortField', val: string): void
  (e: 'update:sortOrder', val: 'asc' | 'desc'): void
  (e: 'update:isGridView', val: boolean): void
}>()

function toggleSort() {
  emit('update:sortOrder', props.sortOrder === 'asc' ? 'desc' : 'asc')
}
</script>

<template>
  <div class="search-sort-bar">
    <div class="search-wrapper">
      <IconSearch class="search-icon" />
      <input
        :value="searchQuery"
        @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
        type="text"
        :placeholder="searchPlaceholder || 'Search...'"
        class="search-input"
      />
      <button v-if="searchQuery" class="clear-search" @click="emit('update:searchQuery', '')">×</button>
    </div>

    <div class="sort-controls">
      <!-- Grid/List Toggle Button (optional) -->
      <button
        v-if="isGridView !== undefined"
        class="layout-toggle-btn"
        @click="emit('update:isGridView', !isGridView)"
        :title="isGridView ? 'Switch to List View' : 'Switch to Grid View'"
      >
        <IconList v-if="isGridView" class="layout-icon" />
        <IconGrid v-else class="layout-icon" />
      </button>

      <!-- Custom Sort Field Dropdown (optional) -->
      <CustomSelect
        v-if="sortField !== undefined && sortLabels"
        :model-value="sortField"
        @update:model-value="emit('update:sortField', $event)"
        :options="sortLabels"
        aria-label="Select Sort Field"
      />

      <!-- Sort Direction Button -->
      <button
        class="sort-toggle-btn"
        @click="toggleSort"
        :title="`Sort Direction: ${sortOrder === 'asc' ? 'Ascending' : 'Descending'}`"
      >
        <IconSort
          class="sort-icon"
          :class="{ reversed: sortOrder === 'desc' }"
        />
      </button>
    </div>
  </div>
</template>

<style scoped>
.search-sort-bar {
  display: flex;
  gap: var(--spacing-3);
  align-items: center;
  width: 100%;
}

.search-wrapper {
  position: relative;
  flex: 1;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  width: 16px;
  height: 16px;
  color: var(--color-text-muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: var(--spacing-2) var(--spacing-4) var(--spacing-2) 36px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background-color: rgba(15, 23, 42, 0.4);
  color: var(--color-text);
  font-family: inherit;
  font-size: 0.9rem;
  transition: all var(--transition-fast);
}

[data-theme='light'] .search-input {
  background-color: rgba(255, 255, 255, 0.6);
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.15);
}

.clear-search {
  position: absolute;
  right: 10px;
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 1.25rem;
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.sort-controls {
  display: flex;
  gap: var(--spacing-2);
  align-items: center;
}

.layout-toggle-btn,
.sort-toggle-btn {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background-color: rgba(15, 23, 42, 0.4);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

[data-theme='light'] .layout-toggle-btn,
[data-theme='light'] .sort-toggle-btn {
  background-color: rgba(255, 255, 255, 0.6);
}

.layout-toggle-btn:hover,
.sort-toggle-btn:hover {
  color: var(--color-text);
  border-color: var(--color-primary);
  background-color: rgba(255, 255, 255, 0.05);
}

[data-theme='light'] .layout-toggle-btn:hover,
[data-theme='light'] .sort-toggle-btn:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.layout-icon {
  width: 18px;
  height: 18px;
}

.sort-icon {
  width: 14px;
  height: 14px;
  transition: transform var(--transition-normal);
}

.sort-icon.reversed {
  transform: rotate(180deg);
}
</style>
