<script setup lang="ts">
interface CategoryItem {
  category_id: string
  category_name: string
}

defineProps<{
  categories: CategoryItem[]
  selectedId: string
  isLoading: boolean
  layout: 'sidebar' | 'chips'
}>()

const emit = defineEmits<{
  (e: 'select', id: string): void
}>()
</script>

<template>
  <!-- Sidebar Layout (Desktop) -->
  <aside v-if="layout === 'sidebar'" class="categories-sidebar desktop-only">
    <div v-if="isLoading" class="loading-sidebar">
      <div v-for="i in 8" :key="i" class="skeleton-pill"></div>
    </div>
    <div v-else class="categories-list">
      <button
        v-for="cat in categories"
        :key="cat.category_id"
        class="category-btn"
        :class="{ active: selectedId === cat.category_id }"
        @click="emit('select', cat.category_id)"
      >
        <span class="category-name">{{ cat.category_name }}</span>
      </button>
    </div>
  </aside>

  <!-- Chips Layout (Mobile) -->
  <div v-else-if="layout === 'chips'" class="mobile-categories mobile-only">
    <button
      v-for="cat in categories"
      :key="cat.category_id"
      class="chip-btn"
      :class="{ active: selectedId === cat.category_id }"
      @click="emit('select', cat.category_id)"
    >
      {{ cat.category_name }}
    </button>
  </div>
</template>

<style scoped>
/* Desktop Sidebar Styles */
.categories-sidebar {
  width: 260px;
  background-color: rgba(30, 41, 59, 0.4);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  padding: var(--spacing-4);
  flex-shrink: 0;
  height: 100%;
  box-sizing: border-box;
}

[data-theme='light'] .categories-sidebar {
  background-color: rgba(240, 240, 240, 0.4);
}

.categories-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.category-btn {
  display: flex;
  align-items: center;
  width: 100%;
  padding: var(--spacing-3) var(--spacing-4);
  background: transparent;
  border: none;
  color: var(--color-text-muted);
  border-radius: var(--radius-md);
  font-family: inherit;
  font-size: 0.9rem;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.category-btn:hover {
  color: var(--color-text);
  background-color: rgba(255, 255, 255, 0.02);
}

.category-btn.active {
  color: var(--color-primary);
  background-color: rgba(96, 165, 250, 0.08);
  font-weight: 700;
}

[data-theme='light'] .category-btn.active {
  background-color: rgba(59, 130, 246, 0.05);
}

.loading-sidebar {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
}

.skeleton-pill {
  height: 36px;
  background: linear-gradient(90deg, rgba(255,255,255,0.03) 25%, rgba(255,255,255,0.08) 50%, rgba(255,255,255,0.03) 75%);
  background-size: 200% 100%;
  animation: loading 1.5s infinite;
  border-radius: var(--radius-md);
}

@keyframes loading {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* Mobile Chips Styles */
.mobile-categories {
  display: flex;
  gap: var(--spacing-2);
  overflow-x: auto;
  padding-bottom: 2px;
  scrollbar-width: none;
}

.mobile-categories::-webkit-scrollbar {
  display: none;
}

.chip-btn {
  padding: 6px var(--spacing-3);
  border-radius: 9999px;
  border: 1px solid var(--color-border);
  background-color: rgba(15, 23, 42, 0.3);
  color: var(--color-text-muted);
  font-family: inherit;
  font-size: 0.8rem;
  font-weight: 600;
  white-space: nowrap;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.chip-btn.active {
  background-color: var(--color-primary);
  border-color: var(--color-primary);
  color: #fff;
}

/* Responsive display filters matching app global layouts */
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
