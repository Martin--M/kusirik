<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import type { LiveStream } from '@/types/stream'
import ChannelRow from '@/components/live/ChannelRow.vue'

const props = defineProps<{
  streams: LiveStream[]
  selectedStreamId: number | null
}>()

defineEmits<{
  (e: 'select', stream: LiveStream): void
  (e: 'play', stream: LiveStream): void
}>()

const parentRef = ref<HTMLElement | null>(null)

const rowVirtualizer = useVirtualizer(
  computed(() => ({
    count: props.streams.length,
    getScrollElement: () => parentRef.value,
    estimateSize: () => 66, // height of ChannelRow in px
    overscan: 10,
  }))
)
</script>

<template>
  <div ref="parentRef" class="virtual-list-container">
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
        :style="{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '100%',
          height: `${virtualRow.size}px`,
          transform: `translateY(${virtualRow.start}px)`,
        }"
      >
        <ChannelRow
          :stream="streams[virtualRow.index]"
          :index="virtualRow.index"
          :is-selected="selectedStreamId === streams[virtualRow.index].stream_id"
          @select="$emit('select', $event)"
          @play="$emit('play', $event)"
        />
      </div>
    </div>
    <div v-else class="empty-state">
      <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M4 5a1 1 0 011-1h14a1 1 0 011 1v10a1 1 0 01-1 1H5a1 1 0 01-1-1V5zm1 1v8h14V6H5z M12 16v2 M7 18h10" />
      </svg>
      <p>No channels found matching the filters.</p>
    </div>
  </div>
</template>

<style scoped>
.virtual-list-container {
  flex: 1;
  width: 100%;
  overflow-y: auto;
  min-height: 0;
  position: relative;
  /* Smooth scrolling on mobile devices */
  -webkit-overflow-scrolling: touch;
}

.scroll-wrapper {
  overflow: hidden;
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
