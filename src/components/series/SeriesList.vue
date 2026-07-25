<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import type { Series } from '@/types/series'
import SeriesCard from './SeriesCard.vue'
import ListRowItem from '@/components/ui/ListRowItem.vue'

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
          <ListRowItem
            :name="chunkedItems[virtualRow.index][0].name"
            :image="chunkedItems[virtualRow.index][0].cover"
            :rating="formatRating(chunkedItems[virtualRow.index][0].rating)"
            :year="chunkedItems[virtualRow.index][0].release_date ? new Date((chunkedItems[virtualRow.index][0].release_date as number) * 1000).getUTCFullYear().toString() : undefined"
            :is-selected="String(selectedSeriesId) === String(chunkedItems[virtualRow.index][0].series_id)"
            @select="emit('select', chunkedItems[virtualRow.index][0])"
            @play="emit('play', chunkedItems[virtualRow.index][0])"
          />
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

.empty-list {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-8) var(--spacing-4);
  color: var(--color-text-muted);
  font-size: 0.9rem;
}
</style>
