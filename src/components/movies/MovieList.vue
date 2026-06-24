<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import type { VodStream } from '@/types/vod'
import MovieCard from './MovieCard.vue'
import CachedImage from '@/components/ui/CachedImage.vue'

const props = defineProps<{
  streams: VodStream[]
  selectedStreamId: number | null
  isGridView: boolean
}>()

const emit = defineEmits<{
  (e: 'select', movie: VodStream): void
  (e: 'play', movie: VodStream): void
}>()

const parentRef = ref<HTMLElement | null>(null)
const containerWidth = ref(1000)
let resizeObserver: ResizeObserver | null = null

const formatRating = (ratingStr?: string | null) => {
  if (!ratingStr) return null
  const r = parseFloat(ratingStr)
  return isNaN(r) || r === 0 ? null : r.toFixed(1)
}

// Responsive columns selector
const columns = computed(() => {
  if (!props.isGridView) return 1
  const w = containerWidth.value
  if (w < 400) return 2
  if (w < 600) return 3
  if (w < 850) return 4
  return 5
})

// Chunk items into rows for virtualizer
const chunkedItems = computed(() => {
  const list = props.streams
  const cols = columns.value
  const chunks: VodStream[][] = []
  for (let i = 0; i < list.length; i += cols) {
    chunks.push(list.slice(i, i + cols))
  }
  return chunks
})

const rowHeight = computed(() => {
  if (!props.isGridView) return 72
  // Poster aspect ratio is 2:3.
  // Card height = (containerWidth / columns) * 1.5 + footer info height (~65px)
  const colWidth = containerWidth.value / columns.value
  return colWidth * 1.5 + 75
})

const rowVirtualizer = useVirtualizer(
  computed(() => ({
    count: chunkedItems.value.length,
    getScrollElement: () => parentRef.value,
    estimateSize: () => rowHeight.value,
    overscan: 3,
  }))
)

// Watch isGridView and rowHeight to trigger layout measurements recalculations on resize or layout toggle
watch([() => props.isGridView, columns, rowHeight], () => {
  rowVirtualizer.value.measure()
})

onMounted(() => {
  if (parentRef.value && typeof window !== 'undefined' && 'ResizeObserver' in window) {
    resizeObserver = new ResizeObserver((entries) => {
      if (entries[0] && entries[0].contentRect) {
        containerWidth.value = entries[0].contentRect.width
      }
    })
    resizeObserver.observe(parentRef.value)
  }
})

onBeforeUnmount(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
})
</script>

<template>
  <div ref="parentRef" class="movie-list-container">
    <div
      v-if="streams.length > 0"
      class="scroll-wrapper"
      :style="{
        height: `${rowVirtualizer.getTotalSize()}px`,
        width: '100%',
        position: 'relative',
      }"
    >
      <div
        v-for="virtualRow in rowVirtualizer.getVirtualItems()"
        :key="virtualRow.index"
        class="virtual-row"
        :class="{ 'grid-mode': isGridView }"
        :style="{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '100%',
          height: `${virtualRow.size}px`,
          transform: `translateY(${virtualRow.start}px)`,
        }"
      >
        <template v-if="isGridView">
          <div
            class="grid-row-inner"
            :style="{
              display: 'grid',
              gridTemplateColumns: `repeat(${columns}, 1fr)`,
              gap: 'var(--spacing-3)',
              padding: '0 var(--spacing-4) var(--spacing-3) var(--spacing-4)',
              height: '100%',
            }"
          >
            <MovieCard
              v-for="movie in chunkedItems[virtualRow.index]"
              :key="movie.stream_id"
              :movie="movie"
              :is-selected="selectedStreamId === movie.stream_id"
              @select="emit('select', $event)"
              @play="emit('play', $event)"
            />
          </div>
        </template>
        <template v-else>
          <div
            class="list-row-inner"
            :class="{ selected: selectedStreamId === chunkedItems[virtualRow.index][0].stream_id }"
            @click="emit('select', chunkedItems[virtualRow.index][0])"
            @dblclick="emit('play', chunkedItems[virtualRow.index][0])"
          >
            <div class="list-poster">
              <CachedImage
                :src="chunkedItems[virtualRow.index][0].stream_icon"
                :alt="chunkedItems[virtualRow.index][0].name || ''"
                :fallback-text="chunkedItems[virtualRow.index][0].name || ''"
              />
            </div>
            <div class="list-details">
              <span class="list-title">{{ chunkedItems[virtualRow.index][0].name }}</span>
              <span v-if="formatRating(chunkedItems[virtualRow.index][0].rating)" class="list-rating">
                <svg class="list-star-icon" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" />
                </svg>
                <span>{{ formatRating(chunkedItems[virtualRow.index][0].rating) }}</span>
              </span>
            </div>
            <button
              class="list-play-btn"
              @click.stop="emit('play', chunkedItems[virtualRow.index][0])"
              aria-label="Play Movie"
            >
              <svg viewBox="0 0 24 24" fill="currentColor" class="play-icon">
                <path d="M8 5v14l11-7z" />
              </svg>
            </button>
          </div>
        </template>
      </div>
    </div>
    <div v-else class="empty-state">
      <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M7 4v16M17 4v16M3 8h18M3 16h18" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <p>No movies found matching your filters.</p>
    </div>
  </div>
</template>

<style scoped>
.movie-list-container {
  flex: 1;
  width: 100%;
  overflow-y: auto;
  min-height: 0;
  position: relative;
  -webkit-overflow-scrolling: touch;
}

.scroll-wrapper {
  overflow: hidden;
}

.virtual-row {
  box-sizing: border-box;
}

/* List view styling */
.list-row-inner {
  display: flex;
  align-items: center;
  gap: var(--spacing-4);
  padding: var(--spacing-2) var(--spacing-4);
  margin: 0 var(--spacing-4) var(--spacing-2) var(--spacing-4);
  background-color: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  height: 60px; /* virtual row size 72 allows 12px gap bottom */
  cursor: pointer;
  transition: all var(--transition-fast) ease;
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
  height: 48px;
  border-radius: var(--radius-sm);
  overflow: hidden;
  background-color: rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
}

.list-img, .list-img-fallback {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.list-img-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.25rem;
}

.list-details {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.list-title {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.list-rating {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 0.75rem;
  color: #fbbf24;
  font-weight: 600;
  margin-top: 2px;
}

.list-star-icon {
  width: 12px;
  height: 12px;
  fill: currentColor;
}

.list-play-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background-color: var(--color-primary);
  border: none;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform var(--transition-fast) ease;
}

.list-play-btn:hover {
  transform: scale(1.1);
  background-color: #3b82f6;
}

.play-icon {
  width: 16px;
  height: 16px;
  margin-left: 1px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--spacing-6);
  color: var(--color-text-muted);
  text-align: center;
  gap: var(--spacing-3);
}

.empty-icon {
  width: 48px;
  height: 48px;
  opacity: 0.5;
}

[data-theme='light'] .list-row-inner {
  background-color: rgba(0, 0, 0, 0.01);
}
</style>
