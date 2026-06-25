<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import type { VodStream } from '@/types/vod'
import MovieCard from './MovieCard.vue'
import ListRowItem from '@/components/ui/ListRowItem.vue'
import IconMovies from '@/components/icons/IconMovies.vue'

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

const getMovieYear = (name?: string | null) => {
  if (!name) return null
  const match = name.match(/\((\d{4})\)/)
  return match ? match[1] : null
}

const getMovieCleanTitle = (name?: string | null) => {
  if (!name) return 'Untitled'
  return name.replace(/\s*\(\d{4}\)/, '').trim()
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
          <ListRowItem
            :name="getMovieCleanTitle(chunkedItems[virtualRow.index][0].name)"
            :image="chunkedItems[virtualRow.index][0].stream_icon"
            :rating="formatRating(chunkedItems[virtualRow.index][0].rating)"
            :year="getMovieYear(chunkedItems[virtualRow.index][0].name)"
            :is-selected="String(selectedStreamId) === String(chunkedItems[virtualRow.index][0].stream_id)"
            @select="emit('select', chunkedItems[virtualRow.index][0])"
            @play="emit('play', chunkedItems[virtualRow.index][0])"
          />
        </template>
      </div>
    </div>
    <div v-else class="empty-state">
      <IconMovies class="empty-icon" stroke-width="1.5" />
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

.virtual-row:not(.grid-mode) {
  padding: 0 var(--spacing-4) var(--spacing-2) var(--spacing-4);
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
</style>
