<script setup lang="ts">
import { computed } from 'vue'
import StreamDetailPanel from '@/components/ui/StreamDetailPanel.vue'
import IconStar from '@/components/icons/IconStar.vue'
import IconTVGrid from '@/components/icons/IconTVGrid.vue'
import { useVodInfo } from '@/composables/useVodStreams'
import type { VodStream } from '@/types/vod'

const props = defineProps<{
  stream: VodStream | null
  isMobileOpen: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'play', stream: VodStream): void
  (e: 'copy', stream: VodStream): void
  (e: 'toggle-favorite', stream: VodStream): void
}>()

// On-demand details query when a movie is selected
const selectedMovieId = computed(() => props.stream?.stream_id || 0)
const { data: movieInfo, isLoading: isLoadingInfo } = useVodInfo(selectedMovieId)
</script>

<template>
  <StreamDetailPanel
    :stream="stream"
    :is-mobile-open="isMobileOpen"
    :play-button-text="$t('media.play')"
    :copy-button-text="$t('media.copyUrl')"
    @close="emit('close')"
    @play="emit('play', stream!)"
    @copy="emit('copy', stream!)"
  >
    <template #header-meta>
      <div class="movie-header-meta-row">
        <span v-if="stream?.rating && parseFloat(stream.rating) > 0" class="rating-text-chip">
          <IconStar style="width: 12px; height: 12px; display: inline-block; vertical-align: -1px; margin-right: 4px;" />
          <span>{{ parseFloat(stream.rating).toFixed(1) }}</span>
        </span>
        <button class="btn-fav" :class="{ favorited: !!stream?.is_favorite }" @click="emit('toggle-favorite', stream!)">
          <IconStar class="fav-icon" />
          <span>{{ stream?.is_favorite ? 'Favorited' : 'Favorite' }}</span>
        </button>
      </div>
    </template>

    <template #header-meta-mobile>
      <div class="movie-header-meta-row">
        <span v-if="stream?.rating && parseFloat(stream.rating) > 0" class="rating-text-chip">
          <IconStar style="width: 12px; height: 12px; display: inline-block; vertical-align: -1px; margin-right: 4px;" />
          <span>{{ parseFloat(stream.rating).toFixed(1) }}</span>
        </span>
        <button class="btn-fav" :class="{ favorited: !!stream?.is_favorite }" @click="emit('toggle-favorite', stream!)">
          <IconStar class="fav-icon" />
          <span>{{ stream?.is_favorite ? 'Favorited' : 'Favorite' }}</span>
        </button>
      </div>
    </template>

    <!-- Metadata info loading / loaded -->
    <div class="movie-metadata-box">
      <div v-if="isLoadingInfo" class="metadata-loading">
        <div class="skeleton-meta-line"></div>
        <div class="skeleton-meta-line short"></div>
        <div class="skeleton-meta-line"></div>
      </div>
      <div v-else-if="movieInfo" class="metadata-content">
        <div class="meta-row" v-if="movieInfo.info?.releasedate">
          <span class="meta-label">{{ $t('media.releaseDate') }}:</span>
          <span class="meta-value">{{ movieInfo.info.releasedate }}</span>
        </div>
        <div class="meta-row" v-if="movieInfo.info?.duration || movieInfo.info?.episode_run_time">
          <span class="meta-label">{{ $t('media.duration') }}:</span>
          <span class="meta-value">
            {{ movieInfo.info.duration || `${movieInfo.info.episode_run_time} mins` }}
          </span>
        </div>
        <div class="meta-row" v-if="movieInfo.info?.genre">
          <span class="meta-label">{{ $t('media.genre') }}:</span>
          <span class="meta-value">{{ movieInfo.info.genre }}</span>
        </div>
        <div class="meta-row" v-if="movieInfo.info?.director">
          <span class="meta-label">{{ $t('media.director') }}:</span>
          <span class="meta-value">{{ movieInfo.info.director }}</span>
        </div>
        <div class="meta-row" v-if="movieInfo.info?.cast || movieInfo.info?.actors">
          <span class="meta-label">{{ $t('media.cast') }}:</span>
          <span class="meta-value text-clamp" :title="movieInfo.info.cast || movieInfo.info.actors">
            {{ movieInfo.info.cast || movieInfo.info.actors }}
          </span>
        </div>
        <div class="meta-plot" v-if="movieInfo.info?.plot || movieInfo.info?.description">
          <h4 class="section-subtitle">{{ $t('media.plot') }}</h4>
          <p class="plot-text">{{ movieInfo.info.plot || movieInfo.info.description }}</p>
        </div>
      </div>
      <div v-else class="metadata-empty">
        <p>{{ $t('media.empty') }}</p>
      </div>
    </div>

    <template #no-selection>
      <div class="no-selection">
        <div class="tv-art">
          <IconTVGrid />
        </div>
        <p>{{ $t('media.selectToView') }}</p>
      </div>
    </template>
  </StreamDetailPanel>
</template>

<style scoped>
.movie-metadata-box {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
}

.metadata-loading {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
  padding: var(--spacing-4) 0;
}

.skeleton-meta-line {
  height: 14px;
  background-color: rgba(255, 255, 255, 0.04);
  border-radius: var(--radius-sm);
  animation: pulse-meta 1.5s infinite;
}

.skeleton-meta-line.short {
  width: 50%;
}

@keyframes pulse-meta {
  0% { opacity: 0.4; }
  50% { opacity: 0.8; }
  100% { opacity: 0.4; }
}

.metadata-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
  font-size: 0.85rem;
}

.meta-row {
  display: flex;
  gap: var(--spacing-2);
}

.meta-label {
  color: var(--color-text-muted);
  font-weight: 600;
  min-width: 70px;
}

.meta-value {
  color: var(--color-text);
  font-weight: 500;
}

.text-clamp {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.meta-plot {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
  margin-top: var(--spacing-1);
}

.section-subtitle {
  font-size: 0.85rem;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 700;
  margin: 0;
}

.plot-text {
  font-size: 0.82rem;
  color: var(--color-text-muted);
  line-height: 1.5;
  margin: 0;
}

.metadata-empty {
  color: var(--color-text-muted);
  font-style: italic;
  text-align: center;
  padding: var(--spacing-4);
}

.no-selection {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-8) var(--spacing-6);
  color: var(--color-text-muted);
  text-align: center;
  gap: var(--spacing-4);
  height: 100%;
}

.tv-art {
  width: 64px;
  height: 64px;
  opacity: 0.2;
}

.rating-text-chip {
  font-size: 0.75rem;
  color: #fbbf24;
  background: rgba(251, 191, 36, 0.1);
  padding: 2px 10px;
  border-radius: 9999px;
  font-weight: 700;
}

.movie-header-meta-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  margin-top: var(--spacing-2);
}

.btn-fav {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-1);
  padding: 4px 10px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border);
  background-color: rgba(255, 255, 255, 0.03);
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.btn-fav:hover {
  background-color: rgba(255, 255, 255, 0.08);
  color: var(--color-text);
}

.btn-fav.favorited {
  color: var(--color-primary);
  border-color: var(--color-primary);
  background-color: rgba(59, 130, 246, 0.1);
}

[data-theme='dark'] .btn-fav.favorited {
  background-color: rgba(96, 165, 250, 0.1);
}

.btn-fav.favorited:hover {
  background-color: rgba(59, 130, 246, 0.2);
}

[data-theme='dark'] .btn-fav.favorited:hover {
  background-color: rgba(96, 165, 250, 0.2);
}

.fav-icon {
  width: 12px;
  height: 12px;
}
</style>
