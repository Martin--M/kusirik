<script setup lang="ts">
import type { LiveStream } from '@/types/stream'
import CachedImage from '@/components/ui/CachedImage.vue'
import IconClock from '@/components/icons/IconClock.vue'
import IconPlay from '@/components/icons/IconPlay.vue'

defineProps<{
  stream: LiveStream
  isSelected: boolean
  index: number
}>()

defineEmits<{
  (e: 'select', stream: LiveStream): void
  (e: 'play', stream: LiveStream): void
}>()
</script>

<template>
  <div
    class="channel-row"
    :class="{ selected: isSelected }"
    @click="$emit('select', stream)"
    @dblclick="$emit('play', stream)"
  >
    <div class="index-cell">{{ index + 1 }}</div>
    
    <div class="logo-cell">
      <CachedImage
        :src="stream.stream_icon"
        :alt="stream.name || 'Channel Logo'"
        :fallback-text="stream.name || ''"
      />
    </div>

    <div class="info-cell">
      <div class="name-container">
        <span class="channel-name">{{ stream.name || 'Unnamed Channel' }}</span>
        <span v-if="stream.tv_archive === 1" class="archive-badge" title="Archive / Catch-up Available">
          <IconClock class="badge-icon" />
          <span>Catch-up</span>
        </span>
      </div>
      <div class="epg-placeholder">No EPG data available (Sync to load)</div>
    </div>

    <div class="action-cell">
      <button class="play-btn" @click.stop="$emit('play', stream)" title="Play Channel">
        <IconPlay class="play-icon" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.channel-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-4);
  padding: var(--spacing-3) var(--spacing-4);
  background-color: transparent;
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
  user-select: none;
  transition: all var(--transition-fast);
}

.channel-row:hover {
  background-color: rgba(255, 255, 255, 0.02);
}

[data-theme='light'] .channel-row:hover {
  background-color: rgba(0, 0, 0, 0.01);
}

.channel-row.selected {
  background-color: rgba(96, 165, 250, 0.08);
  border-left: 3px solid var(--color-primary);
  padding-left: calc(var(--spacing-4) - 3px);
}

[data-theme='light'] .channel-row.selected {
  background-color: rgba(59, 130, 246, 0.05);
}

.index-cell {
  width: 32px;
  font-size: 0.85rem;
  color: var(--color-text-muted);
  text-align: right;
  font-weight: 600;
}

.logo-cell {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-md);
  overflow: hidden;
  flex-shrink: 0;
}

.info-cell {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.name-container {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  min-width: 0;
}

.channel-name {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.archive-badge {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.2);
  color: #4ade80;
  font-size: 0.7rem;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 9999px;
  flex-shrink: 0;
}

[data-theme='light'] .archive-badge {
  background: rgba(22, 163, 74, 0.1);
  color: #16a34a;
}

.badge-icon {
  width: 10px;
  height: 10px;
}

.epg-placeholder {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.action-cell {
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.channel-row:hover .action-cell,
.channel-row.selected .action-cell {
  opacity: 1;
}

.play-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background-color: var(--color-primary);
  border: none;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: 0 2px 6px rgba(96, 165, 250, 0.3);
  transition: all var(--transition-fast);
}

.play-btn:hover {
  transform: scale(1.1);
  background-color: var(--color-primary-hover);
}

.play-icon {
  width: 14px;
  height: 14px;
  margin-left: 2px;
}
</style>
