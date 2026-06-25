<script setup lang="ts">
import { computed } from 'vue'
import type { VodStream } from '@/types/vod'
import CachedImage from '@/components/ui/CachedImage.vue'
import IconStar from '@/components/icons/IconStar.vue'
import IconPlay from '@/components/icons/IconPlay.vue'

const props = defineProps<{
  movie: VodStream
  isSelected: boolean
}>()

defineEmits<{
  (e: 'select', movie: VodStream): void
  (e: 'play', movie: VodStream): void
}>()

// Extract year from movie title like "Movie Name (2020)"
const year = computed(() => {
  if (!props.movie.name) return null
  const match = props.movie.name.match(/\((\d{4})\)/)
  return match ? match[1] : null
})

// Clean up title (remove year parenthesis if present)
const cleanTitle = computed(() => {
  if (!props.movie.name) return ''
  return props.movie.name.replace(/\s*\(\d{4}\)/, '').trim()
})

const displayRating = computed(() => {
  if (!props.movie.rating) return null
  const r = parseFloat(props.movie.rating)
  return isNaN(r) || r === 0 ? null : r.toFixed(1)
})
</script>

<template>
  <div
    class="movie-card"
    :class="{ selected: isSelected }"
    @click="$emit('select', movie)"
    @dblclick="$emit('play', movie)"
  >
    <div class="poster-wrapper">
      <CachedImage
        :src="movie.stream_icon"
        :alt="movie.name || 'Movie'"
        :fallback-text="cleanTitle"
      />
      <div v-if="displayRating" class="rating-badge">
        <IconStar class="star-icon" />
        <span>{{ displayRating }}</span>
      </div>
      <div class="card-overlay">
        <button class="play-overlay-btn" @click.stop="$emit('play', movie)" aria-label="Play Movie">
          <IconPlay class="play-icon" />
        </button>
      </div>
    </div>
    <div class="movie-info">
      <h3 class="movie-title" :title="movie.name || ''">{{ cleanTitle }}</h3>
      <span v-if="year" class="movie-year">{{ year }}</span>
    </div>
  </div>
</template>

<style scoped>
.movie-card {
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

.movie-card:hover {
  transform: translateY(-4px);
  border-color: var(--color-primary);
  background-color: rgba(255, 255, 255, 0.04);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}

.movie-card.selected {
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

.movie-card:hover .card-overlay {
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

.play-icon {
  width: 24px;
  height: 24px;
  margin-left: 2px;
}

.movie-info {
  padding: var(--spacing-3);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-1);
}

.movie-title {
  margin: 0;
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.movie-year {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

[data-theme='light'] .movie-card {
  background-color: rgba(0, 0, 0, 0.02);
}

[data-theme='light'] .movie-card:hover {
  background-color: rgba(0, 0, 0, 0.03);
}
</style>
