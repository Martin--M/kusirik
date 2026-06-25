<script setup lang="ts">
import CachedImage from './CachedImage.vue'
import IconStar from '../icons/IconStar.vue'
import IconArrowRight from '../icons/IconArrowRight.vue'

defineProps<{
  name?: string | null
  image?: string | null
  rating?: string | null
  year?: string | number | null
  isSelected: boolean
}>()

defineEmits<{
  (e: 'select'): void
  (e: 'play'): void
}>()
</script>

<template>
  <div
    class="list-row-inner"
    :class="{ selected: isSelected }"
    @click="$emit('select')"
    @dblclick="$emit('play')"
  >
    <div class="list-poster">
      <CachedImage
        :src="image"
        :alt="name || 'Cover'"
        :fallback-text="name || ''"
      />
    </div>
    <div class="list-details">
      <h3 class="list-title" :title="name || ''">{{ name || 'Untitled' }}</h3>
      <div class="list-meta">
        <span v-if="rating" class="list-rating">
          <IconStar class="star-icon" />
          <span>{{ rating }}</span>
        </span>
        <span v-if="year" class="list-year">{{ year }}</span>
      </div>
    </div>
    <button class="list-play-btn" @click.stop="$emit('play')" aria-label="Open Details">
      <IconArrowRight class="play-icon" />
    </button>
  </div>
</template>

<style scoped>
.list-row-inner {
  display: flex;
  align-items: center;
  width: 100%;
  height: 100%;
  background-color: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-2) var(--spacing-4);
  cursor: pointer;
  gap: var(--spacing-4);
  transition: all var(--transition-fast) ease;
  user-select: none;
}

.list-row-inner:hover {
  border-color: var(--color-primary);
  background-color: rgba(255, 255, 255, 0.04);
}

.list-row-inner.selected {
  border-color: var(--color-primary);
  background-color: rgba(96, 165, 250, 0.08);
}

.list-poster {
  width: 40px;
  height: 56px;
  border-radius: var(--radius-sm);
  overflow: hidden;
  background-color: rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
}

.list-details {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-1);
  min-width: 0;
}

.list-title {
  margin: 0;
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.list-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  font-size: 0.75rem;
}

.list-rating {
  color: #fbbf24;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 2px;
}

.star-icon {
  width: 12px;
  height: 12px;
}

.list-year {
  color: var(--color-text-muted);
}

.list-play-btn {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: var(--spacing-2);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color var(--transition-fast) ease;
}

.list-row-inner:hover .list-play-btn {
  color: var(--color-primary);
}

.play-icon {
  width: 20px;
  height: 20px;
}

[data-theme='light'] .list-row-inner {
  background-color: rgba(0, 0, 0, 0.01);
}

[data-theme='light'] .list-row-inner:hover {
  background-color: rgba(0, 0, 0, 0.02);
}
</style>
