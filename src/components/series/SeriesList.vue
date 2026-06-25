<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import type { Series } from '@/types/series'
import SeriesCard from './SeriesCard.vue'
import CachedImage from '@/components/ui/CachedImage.vue'

const props = defineProps<{
  seriesList: Series[]
  selectedSeriesId: number | null
  isGridView: boolean
}>()

const emit = defineEmits<{
  (e: 'select', series: Series): void
  (e: 'play', series: Series): void
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
  const list = props.seriesList
  const cols = columns.value
  const chunks: Series[][] = []
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

// Watch isGridView and rowHeight to trigger layout measurements recalculations
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
  <div ref="parentRef" class="series-list-container">
    <div
      v-if="seriesList.length > 0"
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
            <SeriesCard
              v-for="series in chunkedItems[virtualRow.index]"
              :key="series.series_id"
              :series="series"
              :is-selected="String(selectedSeriesId) === String(series.series_id)"
              @select="emit('select', $event)"
              @play="emit('play', $event)"
            />
          </div>
        </template>
        <template v-else>
          <div
            class="list-row-inner"
            :class="{ selected: String(selectedSeriesId) === String(chunkedItems[virtualRow.index][0].series_id) }"
            @click="emit('select', chunkedItems[virtualRow.index][0])"
            @dblclick="emit('play', chunkedItems[virtualRow.index][0])"
          >
            <div class="list-poster">
              <CachedImage
                :src="chunkedItems[virtualRow.index][0].cover"
                :alt="chunkedItems[virtualRow.index][0].name || ''"
                :fallback-text="chunkedItems[virtualRow.index][0].name || ''"
              />
            </div>
            <div class="list-details">
              <h3 class="list-title">{{ chunkedItems[virtualRow.index][0].name }}</h3>
              <div class="list-meta">
                <span v-if="formatRating(chunkedItems[virtualRow.index][0].rating)" class="list-rating">
                  <svg viewBox="0 0 24 24" fill="currentColor" style="width: 12px; height: 12px; display: inline-block; vertical-align: -1px; margin-right: 2px;">
                    <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" />
                  </svg>
                  <span>{{ formatRating(chunkedItems[virtualRow.index][0].rating) }}</span>
                </span>
                <span v-if="chunkedItems[virtualRow.index][0].release_date" class="list-year">
                  {{ chunkedItems[virtualRow.index][0].release_date?.split('-')[0] }}
                </span>
              </div>
            </div>
            <button class="list-play-btn" @click.stop="emit('play', chunkedItems[virtualRow.index][0])" aria-label="Open details">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="play-icon">
                <path d="M5 12h14M12 5l7 7-7 7" />
              </svg>
            </button>
          </div>
        </template>
      </div>
    </div>
    <div v-else class="empty-list">
      <p>No TV series found matching the criteria.</p>
    </div>
  </div>
</template>

<style scoped>
.series-list-container {
  flex: 1;
  width: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  min-height: 0;
  position: relative;
}

.scroll-wrapper {
  will-change: transform;
}

.virtual-row {
  display: flex;
  width: 100%;
}

.virtual-row:not(.grid-mode) {
  padding: 0 var(--spacing-4) var(--spacing-2) var(--spacing-4);
}

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

.empty-list {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-8) var(--spacing-4);
  color: var(--color-text-muted);
  font-size: 0.9rem;
}

[data-theme='light'] .list-row-inner {
  background-color: rgba(0, 0, 0, 0.01);
}

[data-theme='light'] .list-row-inner:hover {
  background-color: rgba(0, 0, 0, 0.02);
}
</style>
