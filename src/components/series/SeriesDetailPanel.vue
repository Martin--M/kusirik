<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import StreamDetailPanel from '@/components/ui/StreamDetailPanel.vue'
import IconStar from '@/components/icons/IconStar.vue'
import IconPlay from '@/components/icons/IconPlay.vue'
import IconCopy from '@/components/icons/IconCopy.vue'
import IconChevron from '@/components/icons/IconChevron.vue'
import IconTVPlus from '@/components/icons/IconTVPlus.vue'
import { useSeriesInfo } from '@/composables/useSeries'
import { usePlayer } from '@/composables/usePlayer'
import { useProfileStore } from '@/stores/profile.store'
import { useToastStore } from '@/stores/toast.store'
import { useI18n } from '@/composables/useI18n'
import { getSetting, copyToSystemClipboard } from '@/lib/tauri-commands'
import { buildEpisodeUrl } from '@/lib/url-builder'
import type { Series } from '@/types/series'

const props = defineProps<{
  series: Series | null
  isMobileOpen: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'toggle-favorite', series: Series): void
}>()

const { t } = useI18n()
const toastStore = useToastStore()
const profileStore = useProfileStore()
const { playEpisode } = usePlayer()

const expandedSeason = ref<string | number | null>(null)

// Computed series ID for details query
const selectedSeriesId = computed(() => props.series?.series_id || 0)
const { data: seriesDetails, isLoading: isLoadingDetails } = useSeriesInfo(selectedSeriesId)

// Map selected series structure to StreamDetailPanel format
const mappedSelectedSeries = computed(() => {
  if (!props.series) return null
  return {
    ...props.series,
    stream_id: props.series.series_id,
    name: props.series.name,
    stream_icon: props.series.cover
  }
})

// Auto-expand first season when details load
watch(seriesDetails, (newDetails) => {
  if (newDetails && newDetails.episodes) {
    const keys = Object.keys(newDetails.episodes)
    if (keys.length > 0) {
      expandedSeason.value = keys[0]
    }
  } else {
    expandedSeason.value = null
  }
})

function toggleSeason(seasonKey: string | number) {
  if (expandedSeason.value === seasonKey) {
    expandedSeason.value = null
  } else {
    expandedSeason.value = seasonKey
  }
}

function handlePlayEpisode(episode: any) {
  const ext = episode.container_extension || 'mp4'
  const episodeStreamId = typeof episode.id === 'string' ? parseInt(episode.id, 10) : episode.id
  playEpisode(episodeStreamId, ext, props.series?.series_id)
}

async function copyEpisodeUrl(episode: any) {
  try {
    const profile = profileStore.profile
    if (!profile) {
      toastStore.showToast(t('settings.profile.disconnectFailed', { error: 'No profile' }), 'error')
      return
    }

    const password = await getSetting('password')
    if (!password) {
      toastStore.showToast(t('setup.saveFailed', { error: 'Credentials' }), 'error')
      return
    }

    const ext = episode.container_extension || 'mp4'
    const episodeStreamId = typeof episode.id === 'string' ? parseInt(episode.id, 10) : episode.id
    
    const url = buildEpisodeUrl(
      {
        serverUrl: profile.server_url,
        username: profile.username,
        password,
      },
      episodeStreamId,
      ext
    )

    await copyToSystemClipboard(url)
    toastStore.showToast(t('media.urlCopied'), 'success')
  } catch (e) {
    console.error(e)
    toastStore.showToast(t('media.copyFailed'), 'error')
  }
}
</script>

<template>
  <StreamDetailPanel
    :stream="mappedSelectedSeries"
    :is-mobile-open="isMobileOpen"
    :show-actions="false"
    @close="emit('close')"
  >
    <template #header-meta>
      <div class="series-header-meta-row">
        <span v-if="series?.rating && parseFloat(series.rating) > 0" class="rating-text-chip">
          <IconStar style="width: 12px; height: 12px; display: inline-block; vertical-align: -1px; margin-right: 4px;" />
          <span>{{ parseFloat(series.rating).toFixed(1) }}</span>
        </span>
        <button class="btn-fav" :class="{ favorited: !!series?.is_favorite }" @click="emit('toggle-favorite', series!)">
          <IconStar class="fav-icon" />
          <span>{{ series?.is_favorite ? 'Favorited' : 'Favorite' }}</span>
        </button>
      </div>
    </template>

    <template #header-meta-mobile>
      <div class="series-header-meta-row">
        <span v-if="series?.rating && parseFloat(series.rating) > 0" class="rating-text-chip">
          <IconStar style="width: 12px; height: 12px; display: inline-block; vertical-align: -1px; margin-right: 4px;" />
          <span>{{ parseFloat(series.rating).toFixed(1) }}</span>
        </span>
        <button class="btn-fav" :class="{ favorited: !!series?.is_favorite }" @click="emit('toggle-favorite', series!)">
          <IconStar class="fav-icon" />
          <span>{{ series?.is_favorite ? 'Favorited' : 'Favorite' }}</span>
        </button>
      </div>
    </template>

    <!-- Metadata info loading / loaded + Accordion episodes list -->
    <div class="series-metadata-box">
      <div v-if="isLoadingDetails" class="metadata-loading">
        <div class="skeleton-meta-line"></div>
        <div class="skeleton-meta-line short"></div>
        <div class="skeleton-meta-line"></div>
        <div class="skeleton-meta-line short"></div>
      </div>
      <div v-else-if="seriesDetails" class="metadata-content">
        <!-- Quick Info Cards -->
        <div class="meta-row" v-if="series?.genre || seriesDetails.info?.genre">
          <span class="meta-label">{{ $t('media.genre') }}:</span>
          <span class="meta-value">{{ series?.genre || seriesDetails.info?.genre }}</span>
        </div>
        <div class="meta-row" v-if="series?.director || seriesDetails.info?.director">
          <span class="meta-label">{{ $t('media.director') }}:</span>
          <span class="meta-value">{{ series?.director || seriesDetails.info?.director }}</span>
        </div>
        <div class="meta-row" v-if="series?.cast_ || seriesDetails.info?.cast || seriesDetails.info?.actors">
          <span class="meta-label">{{ $t('media.cast') }}:</span>
          <span class="meta-value text-clamp" :title="series?.cast_ || seriesDetails.info?.cast || seriesDetails.info?.actors">
            {{ series?.cast_ || seriesDetails.info?.cast || seriesDetails.info?.actors }}
          </span>
        </div>
        <div class="meta-plot" v-if="series?.plot || seriesDetails.info?.plot || seriesDetails.info?.description">
          <h4 class="section-subtitle">{{ $t('media.plot') }}</h4>
          <p class="plot-text">{{ series?.plot || seriesDetails.info?.plot || seriesDetails.info?.description }}</p>
        </div>

        <!-- Seasons Accordion Tree -->
        <div class="seasons-accordion" v-if="seriesDetails.episodes && Object.keys(seriesDetails.episodes).length > 0">
          <h4 class="section-subtitle accordion-heading">{{ $t('media.seasons') }} & {{ $t('media.episodes') }}</h4>
          
          <div
            v-for="(episodesList, seasonKey) in seriesDetails.episodes"
            :key="seasonKey"
            class="season-group"
            :class="{ expanded: expandedSeason === seasonKey }"
          >
            <!-- Season Header Trigger -->
            <button class="season-header" @click="toggleSeason(seasonKey)">
              <span class="season-title">{{ $t('media.season', { num: seasonKey }) }}</span>
              <span class="episode-count">{{ episodesList.length }} {{ $t('media.episodes').toLowerCase() }}</span>
              <IconChevron class="chevron-icon" />
            </button>

            <!-- Episodes List -->
            <div class="episodes-container" v-show="expandedSeason === seasonKey">
              <div
                v-for="episode in episodesList"
                :key="episode.id"
                class="episode-row"
              >
                <div class="episode-main">
                  <span class="episode-num">Ep {{ episode.episode_num }}</span>
                  <span class="episode-title" :title="episode.title || ''">
                    {{ episode.title || $t('media.episode', { num: episode.episode_num }) }}
                  </span>
                </div>
                <div class="episode-info" v-if="episode.info?.plot">
                  <p class="episode-plot">{{ episode.info.plot }}</p>
                </div>
                <div class="episode-actions">
                  <button class="action-btn play-btn" @click="handlePlayEpisode(episode)">
                    <IconPlay class="action-icon" />
                    <span>{{ $t('media.play') }}</span>
                  </button>
                  <button class="action-btn copy-btn" @click="copyEpisodeUrl(episode)" :title="$t('media.copyUrl')">
                    <IconCopy class="action-icon" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div v-else class="no-episodes-msg">
          <p>{{ $t('media.empty') }}</p>
        </div>
      </div>
      <div v-else class="metadata-empty">
        <p>{{ $t('media.empty') }}</p>
      </div>
    </div>

    <template #no-selection>
      <div class="no-selection">
        <div class="tv-art">
          <IconTVPlus />
        </div>
        <p>{{ $t('media.selectToView') }}</p>
      </div>
    </template>
  </StreamDetailPanel>
</template>

<style scoped>
.series-metadata-box {
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

.series-header-meta-row {
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

/* Accordion Seasons/Episodes Styles */
.seasons-accordion {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
  margin-top: var(--spacing-4);
}

.accordion-heading {
  margin-bottom: var(--spacing-2);
}

.season-group {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  background-color: rgba(255, 255, 255, 0.01);
  transition: border-color var(--transition-fast) ease;
}

.season-group.expanded {
  border-color: rgba(59, 130, 246, 0.3);
}

.season-header {
  display: flex;
  align-items: center;
  width: 100%;
  padding: var(--spacing-3) var(--spacing-4);
  background: none;
  border: none;
  cursor: pointer;
  text-align: left;
  gap: var(--spacing-3);
  color: var(--color-text);
  transition: background-color var(--transition-fast) ease;
}

.season-header:hover {
  background-color: rgba(255, 255, 255, 0.03);
}

.season-title {
  font-weight: 600;
  font-size: 0.9rem;
  flex: 1;
}

.episode-count {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.chevron-icon {
  width: 16px;
  height: 16px;
  color: var(--color-text-muted);
  transition: transform var(--transition-fast) ease;
}

.expanded .chevron-icon {
  transform: rotate(180deg);
  color: var(--color-primary);
}

.episodes-container {
  display: flex;
  flex-direction: column;
  border-top: 1px solid var(--color-border);
  background-color: rgba(0, 0, 0, 0.15);
}

.episode-row {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-3) var(--spacing-4);
  border-bottom: 1px solid var(--color-border);
  gap: var(--spacing-2);
}

.episode-row:last-child {
  border-bottom: none;
}

.episode-main {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-3);
}

.episode-num {
  font-weight: 700;
  font-size: 0.8rem;
  color: var(--color-primary);
  background-color: rgba(59, 130, 246, 0.1);
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  margin-top: 2px;
}

.episode-title {
  font-weight: 600;
  font-size: 0.85rem;
  color: var(--color-text);
  line-height: 1.4;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.episode-info {
  margin-left: 0;
}

.episode-plot {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  line-height: 1.4;
  margin: 0;
}

.episode-actions {
  display: flex;
  gap: var(--spacing-2);
  margin-top: var(--spacing-1);
  align-self: flex-end;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: var(--spacing-1);
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border);
  background-color: rgba(255, 255, 255, 0.02);
  color: var(--color-text);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
  font-weight: 500;
}

.action-btn:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.action-btn:active {
  transform: scale(0.95);
}

.action-btn.play-btn {
  background-color: var(--color-primary);
  border-color: var(--color-primary);
  color: white;
}

.action-btn.play-btn:hover {
  background-color: var(--color-primary-hover);
}

.action-icon {
  width: 12px;
  height: 12px;
}

.no-episodes-msg {
  color: var(--color-text-muted);
  font-style: italic;
  padding: var(--spacing-4);
  text-align: center;
}
</style>
