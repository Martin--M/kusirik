<script setup lang="ts">
import { computed } from 'vue'
import type { Series } from '@/types/series'
import CachedImage from '@/components/ui/CachedImage.vue'

const props = defineProps<{
  series: Series
  isSelected: boolean
}>()

defineEmits<{
  (e: 'select', series: Series): void
  (e: 'play', series: Series): void
}>()

// Extract year from series name (e.g. "Series Name (2020)") or use release_date
const year = computed(() => {
  if (props.series.name) {
    const match = props.series.name.match(/\((\d{4})\)/)
    if (match) return match[1]
  }
  if (props.series.release_date) {
    const datePart = props.series.release_date.split('-')[0]
    if (datePart && /^\d{4}$/.test(datePart)) return datePart
  }
  return null
})

// Clean up title (remove year parenthesis if present)
const cleanTitle = computed(() => {
  if (!props.series.name) return ''
  return props.series.name.replace(/\s*\(\d{4}\)/, '').trim()
})

const displayRating = computed(() => {
  if (!props.series.rating) return null
  const r = parseFloat(props.series.rating)
  return isNaN(r) || r === 0 ? null : r.toFixed(1)
})
</script>

<template>
  <div
    class="series-card"
    :class="{ selected: isSelected }"
    @click="$emit('select', series)"
    @dblclick="$emit('play', series)"
  >
    <div class="poster-wrapper">
      <CachedImage
        :src="series.cover"
        :alt="series.name || 'Series'"
        :fallback-text="cleanTitle"
      />
      <div v-if="displayRating" class="rating-badge">
        <svg class="star-icon" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" />
        </svg>
        <span>{{ displayRating }}</span>
      </div>
      <div class="card-overlay">
        <button class="play-overlay-btn" @click.stop="$emit('play', series)" aria-label="Open Series Details">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="detail-icon">
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
        </button>
      </div>
    </div>
    <div class="series-info">
      <h3 class="series-title" :title="series.name || ''">{{ cleanTitle }}</h3>
      <span v-if="year" class="series-year">{{ year }}</span>
    </div>
  </div>
</template>

<style scoped>
.series-card {
  display: flex;
  flex-direction: column;
  background-color: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
  user-select: none;
}

.series-card:hover {
  transform: translateY(-4px);
  border-color: var(--color-primary);
  background-color: rgba(255, 255, 255, 0.04);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}

.series-card.selected {
  border-color: var(--color-primary);
  background-color: rgba(96, 165, 250, 0.08);
}

.poster-wrapper {
  position: relative;
  width: 100%;
  aspect-ratio: 2 / 3;
  background-color: rgba(0, 0, 0, 0.2);
}

.rating-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background-color: rgba(15, 23, 42, 0.85);
  backdrop-filter: blur(4px);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: var(--radius-sm);
  color: #fbbf24;
  font-size: 0.75rem;
  font-weight: 700;
  z-index: 2;
}

.star-icon {
  width: 12px;
  height: 12px;
}

.card-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(15, 23, 42, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity var(--transition-fast) ease;
  z-index: 1;
}

.series-card:hover .card-overlay {
  opacity: 1;
}

.play-overlay-btn {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background-color: var(--color-primary);
  border: none;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform var(--transition-fast) ease;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.play-overlay-btn:hover {
  transform: scale(1.1);
  background-color: #3b82f6;
}

.detail-icon {
  width: 24px;
  height: 24px;
}

.series-info {
  padding: var(--spacing-3);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-1);
}

.series-title {
  margin: 0;
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.series-year {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

[data-theme='light'] .series-card {
  background-color: rgba(0, 0, 0, 0.02);
}

[data-theme='light'] .series-card:hover {
  background-color: rgba(0, 0, 0, 0.03);
}
</style>
