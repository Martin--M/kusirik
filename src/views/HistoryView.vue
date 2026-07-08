<script setup lang="ts">
import { ref, computed } from 'vue'
import { useHistory, useRemoveFromHistory, useClearHistory } from '@/composables/useHistory'
import { useToggleFavorite } from '@/composables/useFavorites'
import { usePlayer } from '@/composables/usePlayer'
import { useToastStore } from '@/stores/toast.store'
import { useProfileStore } from '@/stores/profile.store'
import { useI18n } from '@/composables/useI18n'
import { getSetting } from '@/lib/tauri-commands'

import ChannelRow from '@/components/live/ChannelRow.vue'
import MovieCard from '@/components/movies/MovieCard.vue'
import SeriesCard from '@/components/series/SeriesCard.vue'
import LiveDetailPanel from '@/components/live/LiveDetailPanel.vue'
import MovieDetailPanel from '@/components/movies/MovieDetailPanel.vue'
import SeriesDetailPanel from '@/components/series/SeriesDetailPanel.vue'
import IconClock from '@/components/icons/IconClock.vue'
import IconChevron from '@/components/icons/IconChevron.vue'

import type { LiveStream } from '@/types/stream'
import type { VodStream } from '@/types/vod'
import type { Series } from '@/types/series'

const toastStore = useToastStore()
const profileStore = useProfileStore()
const { playLive, playMovie } = usePlayer()
const { t } = useI18n()

const { data: history, isLoading } = useHistory()
const { remove: removeEntry } = useRemoveFromHistory()
const { clear: clearHistoryList } = useClearHistory()
const { toggle: toggleFav } = useToggleFavorite()

// Detail panel selection states
const selectedLive = ref<LiveStream | null>(null)
const selectedMovie = ref<VodStream | null>(null)
const selectedSeries = ref<Series | null>(null)
const isMobileDetailOpen = ref(false)

// Collapsible sections state
const isLiveExpanded = ref(true)
const isMoviesExpanded = ref(true)
const isSeriesExpanded = ref(true)

const hasHistory = computed(() => {
  if (!history.value) return false
  return (
    history.value.live.length > 0 ||
    history.value.vod.length > 0 ||
    history.value.series.length > 0
  )
})

function closeDetails() {
  selectedLive.value = null
  selectedMovie.value = null
  selectedSeries.value = null
  isMobileDetailOpen.value = false
}

function handlePlayLive(stream: LiveStream) {
  playLive(stream.stream_id)
}

function handlePlayMovie(stream: VodStream) {
  playMovie(stream.stream_id, stream.container_extension || 'mp4')
}

async function handleClearHistory() {
  try {
    await clearHistoryList()
    toastStore.showToast(t('history.clearSuccess') || 'Playback history cleared successfully', 'success')
    closeDetails()
  } catch (err) {
    console.error('Failed to clear history:', err)
    toastStore.showToast('Failed to clear playback history', 'error')
  }
}

async function handleDeleteEntry(mediaType: 'live' | 'vod' | 'series', id: number) {
  try {
    await removeEntry(mediaType, id)
    // If the currently viewed details panel item was deleted, close the panel
    if (mediaType === 'live' && selectedLive.value?.stream_id === id) selectedLive.value = null
    if (mediaType === 'vod' && selectedMovie.value?.stream_id === id) selectedMovie.value = null
    if (mediaType === 'series' && selectedSeries.value?.series_id === id) selectedSeries.value = null
  } catch (err) {
    console.error('Failed to remove history item:', err)
    toastStore.showToast('Failed to remove entry from history', 'error')
  }
}

// Copy URLs fallback
async function copyUrl(stream: any, type: 'live' | 'movie') {
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

    const { copyToSystemClipboard } = await import('@/lib/tauri-commands')
    const { buildLiveUrl, buildMovieUrl } = await import('@/lib/url-builder')

    let url = ''
    if (type === 'live') {
      url = buildLiveUrl(
        { serverUrl: profile.server_url, username: profile.username, password },
        stream.stream_id,
        'ts'
      )
    } else {
      url = buildMovieUrl(
        { serverUrl: profile.server_url, username: profile.username, password },
        stream.stream_id,
        stream.container_extension || 'mp4'
      )
    }

    await copyToSystemClipboard(url)
    toastStore.showToast(t('media.urlCopied'), 'success')
  } catch (e: any) {
    toastStore.showToast(t('media.copyFailed'), 'error')
  }
}

async function handleToggleLiveFavorite(stream: LiveStream) {
  const originalState = selectedLive.value?.is_favorite ?? 0
  const nextState = originalState === 1 ? 0 : 1

  if (selectedLive.value && selectedLive.value.stream_id === stream.stream_id) {
    selectedLive.value = { ...selectedLive.value, is_favorite: nextState }
  }

  try {
    const isFav = await toggleFav('live', stream.stream_id)
    if (selectedLive.value && selectedLive.value.stream_id === stream.stream_id) {
      selectedLive.value = { ...selectedLive.value, is_favorite: isFav ? 1 : 0 }
    }
  } catch (err) {
    console.error('Failed to toggle favorite:', err)
    if (selectedLive.value && selectedLive.value.stream_id === stream.stream_id) {
      selectedLive.value = { ...selectedLive.value, is_favorite: originalState }
    }
    toastStore.showToast(t('media.favoriteToggleFailed') || 'Failed to update favorite status', 'error')
  }
}

async function handleToggleMovieFavorite(movie: VodStream) {
  const originalState = selectedMovie.value?.is_favorite ?? 0
  const nextState = originalState === 1 ? 0 : 1

  if (selectedMovie.value && selectedMovie.value.stream_id === movie.stream_id) {
    selectedMovie.value = { ...selectedMovie.value, is_favorite: nextState }
  }

  try {
    const isFav = await toggleFav('vod', movie.stream_id)
    if (selectedMovie.value && selectedMovie.value.stream_id === movie.stream_id) {
      selectedMovie.value = { ...selectedMovie.value, is_favorite: isFav ? 1 : 0 }
    }
  } catch (err) {
    console.error('Failed to toggle favorite:', err)
    if (selectedMovie.value && selectedMovie.value.stream_id === movie.stream_id) {
      selectedMovie.value = { ...selectedMovie.value, is_favorite: originalState }
    }
    toastStore.showToast(t('media.favoriteToggleFailed') || 'Failed to update favorite status', 'error')
  }
}

async function handleToggleSeriesFavorite(series: Series) {
  const originalState = selectedSeries.value?.is_favorite ?? 0
  const nextState = originalState === 1 ? 0 : 1

  if (selectedSeries.value && selectedSeries.value.series_id === series.series_id) {
    selectedSeries.value = { ...selectedSeries.value, is_favorite: nextState }
  }

  try {
    const isFav = await toggleFav('series', series.series_id)
    if (selectedSeries.value && selectedSeries.value.series_id === series.series_id) {
      selectedSeries.value = { ...selectedSeries.value, is_favorite: isFav ? 1 : 0 }
    }
  } catch (err) {
    console.error('Failed to toggle favorite:', err)
    if (selectedSeries.value && selectedSeries.value.series_id === series.series_id) {
      selectedSeries.value = { ...selectedSeries.value, is_favorite: originalState }
    }
    toastStore.showToast(t('media.favoriteToggleFailed') || 'Failed to update favorite status', 'error')
  }
}

function selectLive(stream: LiveStream) {
  closeDetails()
  selectedLive.value = stream
  isMobileDetailOpen.value = true
}

function selectMovie(movie: VodStream) {
  closeDetails()
  selectedMovie.value = movie
  isMobileDetailOpen.value = true
}

function selectSeries(series: Series) {
  closeDetails()
  selectedSeries.value = series
  isMobileDetailOpen.value = true
}
</script>

<template>
  <div class="search-view-container">
    <div class="search-content">
      <!-- Loading State -->
      <div v-if="isLoading" class="search-loading-state">
        <span class="spinner"></span>
        <p>{{ $t('settings.stats.loading') }}</p>
      </div>

      <!-- Empty State -->
      <div v-else-if="!hasHistory" class="search-empty-state">
        <IconClock class="empty-icon" />
        <h3>{{ $t('history.emptyState') }}</h3>
        <p>{{ $t('history.descEmpty') }}</p>
      </div>

      <!-- Playback History Layout -->
      <div v-else class="results-layout">
        <!-- Live TV Section -->
        <section v-if="history?.live.length" class="results-section" :class="{ collapsed: !isLiveExpanded }">
          <button class="section-toggle-btn" @click="isLiveExpanded = !isLiveExpanded" :title="isLiveExpanded ? $t('sidebar.collapse') : $t('sidebar.expand')">
            <h2 class="section-title">{{ $t('history.liveResults', { count: history.live.length }) }}</h2>
            <IconChevron class="chevron-icon" :direction="isLiveExpanded ? 'down' : 'right'" />
          </button>
          <div v-show="isLiveExpanded" class="live-list">
            <ChannelRow
              v-for="(stream, idx) in history.live"
              :key="stream.stream_id"
              :stream="stream"
              :index="idx"
              :is-selected="selectedLive?.stream_id === stream.stream_id"
              show-delete
              @select="selectLive"
              @play="handlePlayLive"
              @delete="handleDeleteEntry('live', stream.stream_id)"
            />
          </div>
        </section>

        <!-- Movies Section -->
        <section v-if="history?.vod.length" class="results-section" :class="{ collapsed: !isMoviesExpanded }">
          <button class="section-toggle-btn" @click="isMoviesExpanded = !isMoviesExpanded" :title="isMoviesExpanded ? $t('sidebar.collapse') : $t('sidebar.expand')">
            <h2 class="section-title">{{ $t('history.movieResults', { count: history.vod.length }) }}</h2>
            <IconChevron class="chevron-icon" :direction="isMoviesExpanded ? 'down' : 'right'" />
          </button>
          <div v-show="isMoviesExpanded" class="media-grid">
            <MovieCard
              v-for="movie in history.vod"
              :key="movie.stream_id"
              :movie="movie"
              :is-selected="selectedMovie?.stream_id === movie.stream_id"
              show-delete
              @select="selectMovie(movie)"
              @play="handlePlayMovie(movie)"
              @delete="handleDeleteEntry('vod', movie.stream_id)"
            />
          </div>
        </section>

        <!-- TV Series Section -->
        <section v-if="history?.series.length" class="results-section" :class="{ collapsed: !isSeriesExpanded }">
          <button class="section-toggle-btn" @click="isSeriesExpanded = !isSeriesExpanded" :title="isSeriesExpanded ? $t('sidebar.collapse') : $t('sidebar.expand')">
            <h2 class="section-title">{{ $t('history.seriesResults', { count: history.series.length }) }}</h2>
            <IconChevron class="chevron-icon" :direction="isSeriesExpanded ? 'down' : 'right'" />
          </button>
          <div v-show="isSeriesExpanded" class="media-grid">
            <SeriesCard
              v-for="item in history.series"
              :key="item.series_id"
              :series="item"
              :is-selected="selectedSeries?.series_id === item.series_id"
              show-delete
              @select="selectSeries(item)"
              @play="selectSeries(item)"
              @delete="handleDeleteEntry('series', item.series_id)"
            />
          </div>
        </section>

        <!-- Clear History Footer -->
        <div class="clear-history-footer">
          <button class="btn-clear-history" @click="handleClearHistory">
            {{ $t('history.clearHistory') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Live Channel Details Panel -->
    <LiveDetailPanel
      :stream="selectedLive"
      :is-mobile-open="isMobileDetailOpen"
      @close="closeDetails"
      @play="handlePlayLive"
      @copy="(stream) => copyUrl(stream, 'live')"
      @toggle-favorite="handleToggleLiveFavorite"
    />

    <!-- Movie Details Panel -->
    <MovieDetailPanel
      :stream="selectedMovie"
      :is-mobile-open="isMobileDetailOpen"
      @close="closeDetails"
      @play="handlePlayMovie"
      @copy="(stream) => copyUrl(stream, 'movie')"
      @toggle-favorite="handleToggleMovieFavorite"
    />

    <!-- TV Series Details Panel -->
    <SeriesDetailPanel
      :series="selectedSeries"
      :is-mobile-open="isMobileDetailOpen"
      @close="closeDetails"
      @toggle-favorite="handleToggleSeriesFavorite"
    />
  </div>
</template>

<style scoped>
.search-view-container {
  display: flex;
  flex-direction: row;
  height: 100%;
  width: 100%;
  overflow: hidden;
  background-color: var(--color-bg);
}

.search-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-6) var(--spacing-8);
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.view-header {
  margin-bottom: var(--spacing-6);
}

.header-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.view-title {
  font-size: 1.75rem;
  font-weight: 800;
  letter-spacing: -0.025em;
  color: var(--color-text);
  margin: 0;
}

.btn-clear-history {
  padding: 8px 16px;
  background-color: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #ef4444;
  border-radius: var(--radius-md);
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.btn-clear-history:hover {
  background-color: rgba(239, 68, 68, 0.2);
  transform: translateY(-1px);
}

.search-empty-state,
.search-loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  text-align: center;
  color: var(--color-text-muted);
  padding: var(--spacing-12);
}

.empty-icon {
  width: 48px;
  height: 48px;
  margin-bottom: var(--spacing-4);
  color: var(--color-primary);
  opacity: 0.8;
}

.search-empty-state h3 {
  font-size: 1.2rem;
  color: var(--color-text);
  margin: 0 0 var(--spacing-2) 0;
}

.search-empty-state p {
  font-size: 0.9rem;
  max-width: 320px;
  line-height: 1.5;
  margin: 0;
}

.results-layout {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-8);
}

.results-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
}

.section-title {
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--color-text);
  margin: 0;
  border-left: 3px solid var(--color-primary);
  padding-left: var(--spacing-3);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.section-toggle-btn {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  width: fit-content;
  text-align: left;
}

.section-toggle-btn .chevron-icon {
  width: 18px;
  height: 18px;
  color: var(--color-text-muted);
  transition: transform var(--transition-fast) ease, color var(--transition-fast) ease;
}

.section-toggle-btn:hover .chevron-icon {
  color: var(--color-primary);
}

.results-section.collapsed {
  opacity: 0.85;
}

.live-list {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  background-color: rgba(255, 255, 255, 0.01);
}

.media-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: var(--spacing-4);
}

.clear-history-footer {
  display: flex;
  justify-content: flex-start;
  margin-top: var(--spacing-6);
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-6);
}
</style>
