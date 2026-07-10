<script setup lang="ts">
import { ref } from 'vue'
import { useFavorites, useToggleFavorite } from '@/composables/useFavorites'
import { usePlayer } from '@/composables/usePlayer'
import { useToastStore } from '@/stores/toast.store'
import { useProfileStore } from '@/stores/profile.store'
import { useI18n } from '@/composables/useI18n'

import IconStar from '@/components/icons/IconStar.vue'
import IconChevron from '@/components/icons/IconChevron.vue'

import ChannelRow from '@/components/live/ChannelRow.vue'
import MovieCard from '@/components/movies/MovieCard.vue'
import SeriesCard from '@/components/series/SeriesCard.vue'
import LiveDetailPanel from '@/components/live/LiveDetailPanel.vue'
import MovieDetailPanel from '@/components/movies/MovieDetailPanel.vue'
import SeriesDetailPanel from '@/components/series/SeriesDetailPanel.vue'

import type { LiveStream } from '@/types/stream'
import type { VodStream } from '@/types/vod'
import type { Series } from '@/types/series'

const toastStore = useToastStore()
const profileStore = useProfileStore()
const { playLive, playMovie } = usePlayer()
const { t } = useI18n()

const { data: favorites, isLoading } = useFavorites()
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

// Watch favorites list changing to update selected states if they are unfavorited
// Favorites auto-close watcher removed to keep panel open on unfavorite, allowing undo.

function closeDetails() {
  selectedLive.value = null
  selectedMovie.value = null
  selectedSeries.value = null
  isMobileDetailOpen.value = false
}

function handlePlayLive(stream: LiveStream) {
  playLive(stream.stream_id, stream.profile_id)
}

function handlePlayMovie(stream: VodStream) {
  playMovie(stream.stream_id, stream.container_extension || 'mp4', stream.profile_id)
}

// Copy URLs fallback
async function copyUrl(stream: any, type: 'live' | 'movie') {
  try {
    const profile = profileStore.profile
    if (!profile) {
      toastStore.showToast(t('settings.profile.disconnectFailed', { error: 'No profile' }), 'error')
      return
    }

    if (type === 'live' && profile.profile_type === 'public_iptv') {
      const { resolveStreamUrl, copyToSystemClipboard } = await import('@/lib/tauri-commands')
      const tempUrl = `https://public/live/${stream.stream_id}.ts`
      const url = await resolveStreamUrl(tempUrl, profile.id)
      await copyToSystemClipboard(url)
      toastStore.showToast(t('media.urlCopied'), 'success')
      return
    }

    const password = profile.password
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
  const originalState = selectedLive.value?.is_favorite ?? 1
  const nextState = originalState === 1 ? 0 : 1

  if (selectedLive.value && selectedLive.value.stream_id === stream.stream_id) {
    selectedLive.value = {
      ...selectedLive.value,
      is_favorite: nextState
    }
  }

  try {
    const isFav = await toggleFav('live', stream.stream_id)
    if (selectedLive.value && selectedLive.value.stream_id === stream.stream_id) {
      selectedLive.value = {
        ...selectedLive.value,
        is_favorite: isFav ? 1 : 0
      }
    }
  } catch (err) {
    console.error('Failed to toggle favorite:', err)
    if (selectedLive.value && selectedLive.value.stream_id === stream.stream_id) {
      selectedLive.value = {
        ...selectedLive.value,
        is_favorite: originalState
      }
    }
    toastStore.showToast(t('media.favoriteToggleFailed') || 'Failed to update favorite status', 'error')
  }
}

async function handleToggleMovieFavorite(movie: VodStream) {
  const originalState = selectedMovie.value?.is_favorite ?? 1
  const nextState = originalState === 1 ? 0 : 1

  if (selectedMovie.value && selectedMovie.value.stream_id === movie.stream_id) {
    selectedMovie.value = {
      ...selectedMovie.value,
      is_favorite: nextState
    }
  }

  try {
    const isFav = await toggleFav('vod', movie.stream_id)
    if (selectedMovie.value && selectedMovie.value.stream_id === movie.stream_id) {
      selectedMovie.value = {
        ...selectedMovie.value,
        is_favorite: isFav ? 1 : 0
      }
    }
  } catch (err) {
    console.error('Failed to toggle favorite:', err)
    if (selectedMovie.value && selectedMovie.value.stream_id === movie.stream_id) {
      selectedMovie.value = {
        ...selectedMovie.value,
        is_favorite: originalState
      }
    }
    toastStore.showToast(t('media.favoriteToggleFailed') || 'Failed to update favorite status', 'error')
  }
}

async function handleToggleSeriesFavorite(series: Series) {
  const originalState = selectedSeries.value?.is_favorite ?? 1
  const nextState = originalState === 1 ? 0 : 1

  if (selectedSeries.value && selectedSeries.value.series_id === series.series_id) {
    selectedSeries.value = {
      ...selectedSeries.value,
      is_favorite: nextState
    }
  }

  try {
    const isFav = await toggleFav('series', series.series_id)
    if (selectedSeries.value && selectedSeries.value.series_id === series.series_id) {
      selectedSeries.value = {
        ...selectedSeries.value,
        is_favorite: isFav ? 1 : 0
      }
    }
  } catch (err) {
    console.error('Failed to toggle favorite:', err)
    if (selectedSeries.value && selectedSeries.value.series_id === series.series_id) {
      selectedSeries.value = {
        ...selectedSeries.value,
        is_favorite: originalState
      }
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
      <div v-else-if="favorites && !favorites.live.length && !favorites.vod.length && !favorites.series.length" class="search-empty-state">
        <IconStar class="empty-icon" />
        <h3>{{ $t('favorites.emptyState') }}</h3>
        <p>{{ $t('favorites.descEmpty') }}</p>
      </div>

      <!-- Favorites List Layout -->
      <div v-else class="results-layout">
        <!-- Live TV Section -->
        <section v-if="favorites?.live.length" class="results-section" :class="{ collapsed: !isLiveExpanded }">
          <button class="section-toggle-btn" @click="isLiveExpanded = !isLiveExpanded" :title="isLiveExpanded ? $t('sidebar.collapse') : $t('sidebar.expand')">
            <h2 class="section-title">{{ $t('favorites.liveResults', { count: favorites.live.length }) }}</h2>
            <IconChevron class="chevron-icon" :direction="isLiveExpanded ? 'down' : 'right'" />
          </button>
          <div v-show="isLiveExpanded" class="live-list">
            <ChannelRow
              v-for="(stream, idx) in favorites.live"
              :key="stream.stream_id"
              :stream="stream"
              :index="idx"
              :is-selected="selectedLive?.stream_id === stream.stream_id"
              @select="selectLive"
              @play="handlePlayLive"
            />
          </div>
        </section>

        <!-- Movies Section -->
        <section v-if="favorites?.vod.length" class="results-section" :class="{ collapsed: !isMoviesExpanded }">
          <button class="section-toggle-btn" @click="isMoviesExpanded = !isMoviesExpanded" :title="isMoviesExpanded ? $t('sidebar.collapse') : $t('sidebar.expand')">
            <h2 class="section-title">{{ $t('favorites.movieResults', { count: favorites.vod.length }) }}</h2>
            <IconChevron class="chevron-icon" :direction="isMoviesExpanded ? 'down' : 'right'" />
          </button>
          <div v-show="isMoviesExpanded" class="media-grid">
            <MovieCard
              v-for="movie in favorites.vod"
              :key="movie.stream_id"
              :movie="movie"
              :is-selected="selectedMovie?.stream_id === movie.stream_id"
              @select="selectMovie(movie)"
              @play="handlePlayMovie(movie)"
            />
          </div>
        </section>

        <!-- TV Series Section -->
        <section v-if="favorites?.series.length" class="results-section" :class="{ collapsed: !isSeriesExpanded }">
          <button class="section-toggle-btn" @click="isSeriesExpanded = !isSeriesExpanded" :title="isSeriesExpanded ? $t('sidebar.collapse') : $t('sidebar.expand')">
            <h2 class="section-title">{{ $t('favorites.seriesResults', { count: favorites.series.length }) }}</h2>
            <IconChevron class="chevron-icon" :direction="isSeriesExpanded ? 'down' : 'right'" />
          </button>
          <div v-show="isSeriesExpanded" class="media-grid">
            <SeriesCard
              v-for="item in favorites.series"
              :key="item.series_id"
              :series="item"
              :is-selected="selectedSeries?.series_id === item.series_id"
              @select="selectSeries(item)"
              @play="selectSeries(item)"
            />
          </div>
        </section>
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

.view-title {
  font-size: 1.75rem;
  font-weight: 800;
  letter-spacing: -0.025em;
  color: var(--color-text);
  margin: 0;
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
  color: #fbbf24;
  opacity: 0.8;
}

.search-empty-state h3 {
  font-size: 1.2rem;
  color: var(--color-text);
  margin: 0 0 var(--spacing-2) 0;
}

.search-empty-state p {
  font-size: 0.9rem;
  max-width: 420px;
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
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: var(--spacing-4);
}

@media (min-width: 640px) {
  .media-grid {
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  }
}

@media (min-width: 1024px) {
  .media-grid {
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  }
}

.live-info-box {
  padding: var(--spacing-4);
  background-color: rgba(255, 255, 255, 0.02);
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-md);
  text-align: center;
  font-size: 0.85rem;
  color: var(--color-text-muted);
}

.live-info-box p {
  margin: 0;
}

.movie-metadata-box,
.series-metadata-box {
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-4);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
}

.meta-row {
  display: flex;
  font-size: 0.85rem;
  line-height: 1.4;
}

.meta-label {
  width: 100px;
  color: var(--color-text-muted);
  font-weight: 600;
  flex-shrink: 0;
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
}

.section-subtitle {
  font-size: 0.85rem;
  text-transform: uppercase;
  color: var(--color-text-muted);
  letter-spacing: 0.05em;
  margin: 0 0 var(--spacing-2) 0;
}

.plot-text {
  font-size: 0.85rem;
  line-height: 1.5;
  color: var(--color-text);
  opacity: 0.9;
  margin: 0;
}

/* Accordion Seasons Styles */
.seasons-accordion {
  margin-top: var(--spacing-4);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
}

.accordion-heading {
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-4);
  margin-bottom: var(--spacing-3);
}

.season-group {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  background: rgba(255, 255, 255, 0.01);
  transition: border-color var(--transition-fast) ease;
}

.season-group.expanded {
  border-color: var(--color-primary);
}

.season-header {
  width: 100%;
  display: flex;
  align-items: center;
  padding: var(--spacing-3) var(--spacing-4);
  background: none;
  border: none;
  color: var(--color-text);
  cursor: pointer;
  font-family: inherit;
  font-size: 0.9rem;
  font-weight: 600;
}

.season-header:hover {
  background: rgba(255, 255, 255, 0.02);
}

.season-title {
  flex: 1;
  text-align: left;
}

.episode-count {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  margin-right: var(--spacing-3);
}

.chevron-icon {
  width: 16px;
  height: 16px;
  color: var(--color-text-muted);
  transition: transform var(--transition-fast) ease;
}

.season-group.expanded .chevron-icon {
  transform: rotate(90deg);
  color: var(--color-primary);
}

.episodes-container {
  display: flex;
  flex-direction: column;
  border-top: 1px solid var(--color-border);
  background: rgba(0, 0, 0, 0.1);
  max-height: 250px;
  overflow-y: auto;
}

.episode-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-3) var(--spacing-4);
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  font-size: 0.8rem;
  transition: background-color var(--transition-fast) ease;
}

.episode-row:hover {
  background-color: rgba(255, 255, 255, 0.02);
}

.episode-main {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  min-width: 0;
  flex: 1;
}

.episode-num {
  font-weight: 700;
  color: var(--color-primary);
}

.episode-title {
  font-weight: 500;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: none;
  background-color: rgba(255, 255, 255, 0.05);
  color: var(--color-text);
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.action-btn:hover {
  background-color: var(--color-primary);
  color: white;
  transform: scale(1.05);
}

.action-icon {
  width: 14px;
  height: 14px;
}
</style>
