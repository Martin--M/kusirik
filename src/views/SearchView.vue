<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useGlobalSearch } from '@/composables/useSearch'
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

import { useToggleFavorite } from '@/composables/useFavorites'

import type { LiveStream } from '@/types/stream'
import type { VodStream } from '@/types/vod'
import type { Series } from '@/types/series'

const route = useRoute()
const toastStore = useToastStore()
const profileStore = useProfileStore()
const { playLive, playMovie } = usePlayer()
const { t } = useI18n()
const { toggle: toggleFav } = useToggleFavorite()

async function handleToggleLiveFavorite(stream: LiveStream) {
  const originalState = selectedLive.value?.is_favorite
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
  const originalState = selectedMovie.value?.is_favorite
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
  const originalState = selectedSeries.value?.is_favorite
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

const queryText = computed(() => (route.query.q as string) || '')
const { data, isLoading } = useGlobalSearch(queryText)


// Detail panel selection states
const selectedLive = ref<LiveStream | null>(null)
const selectedMovie = ref<VodStream | null>(null)
const selectedSeries = ref<Series | null>(null)
const isMobileDetailOpen = ref(false)

// Collapsible sections state
const isLiveExpanded = ref(true)
const isMoviesExpanded = ref(true)
const isSeriesExpanded = ref(true)

// Watch queryText changing to close any open panels
watch(queryText, () => {
  closeDetails()
  isLiveExpanded.value = true
  isMoviesExpanded.value = true
  isSeriesExpanded.value = true
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
      <header class="view-header">
        <h1 class="view-title">{{ $t('search.title') }}</h1>
        <p v-if="queryText" class="search-meta">
          {{ $t('search.showingResults', { query: queryText }) }}
        </p>
      </header>

      <!-- Empty / Idle State -->
      <div v-if="!queryText || queryText.length < 2" class="search-empty-state">
        <IconSearch class="empty-icon" />
        <h3>{{ $t('search.placeholderSearch') }}</h3>
        <p>{{ $t('search.descSearch') }}</p>
      </div>

      <!-- Loading State -->
      <div v-else-if="isLoading" class="search-loading-state">
        <span class="spinner"></span>
        <p>{{ $t('search.searching') }}</p>
      </div>

      <!-- No Results State -->
      <div v-else-if="data && !data.live.length && !data.vod.length && !data.series.length" class="search-empty-state">
        <IconSearch class="empty-icon" />
        <h3>{{ $t('search.noResults', { query: queryText }) }}</h3>
        <p>{{ $t('search.tryDifferent') }}</p>
      </div>

      <!-- Results List -->
      <div v-else class="results-layout">
        <!-- Live TV Section -->
        <section v-if="data?.live.length" class="results-section" :class="{ collapsed: !isLiveExpanded }">
          <button class="section-toggle-btn" @click="isLiveExpanded = !isLiveExpanded" :title="isLiveExpanded ? $t('sidebar.collapse') : $t('sidebar.expand')">
            <h2 class="section-title">{{ $t('search.liveResults', { count: data.live.length }) }}</h2>
            <IconChevron class="chevron-icon" :direction="isLiveExpanded ? 'down' : 'right'" />
          </button>
          <div v-show="isLiveExpanded" class="live-list">
            <ChannelRow
              v-for="(stream, idx) in data.live"
              :key="stream.stream_id"
              :stream="stream as any"
              :index="idx"
              :is-selected="selectedLive?.stream_id === stream.stream_id"
              @select="selectLive(stream)"
              @play="handlePlayLive(stream)"
            />
          </div>
        </section>

        <!-- Movies Section -->
        <section v-if="data?.vod.length" class="results-section" :class="{ collapsed: !isMoviesExpanded }">
          <button class="section-toggle-btn" @click="isMoviesExpanded = !isMoviesExpanded" :title="isMoviesExpanded ? $t('sidebar.collapse') : $t('sidebar.expand')">
            <h2 class="section-title">{{ $t('search.movieResults', { count: data.vod.length }) }}</h2>
            <IconChevron class="chevron-icon" :direction="isMoviesExpanded ? 'down' : 'right'" />
          </button>
          <div v-show="isMoviesExpanded" class="media-grid">
            <MovieCard
              v-for="movie in data.vod"
              :key="movie.stream_id"
              :movie="movie"
              :is-selected="selectedMovie?.stream_id === movie.stream_id"
              @select="selectMovie(movie)"
              @play="handlePlayMovie(movie)"
            />
          </div>
        </section>

        <!-- TV Series Section -->
        <section v-if="data?.series.length" class="results-section" :class="{ collapsed: !isSeriesExpanded }">
          <button class="section-toggle-btn" @click="isSeriesExpanded = !isSeriesExpanded" :title="isSeriesExpanded ? $t('sidebar.collapse') : $t('sidebar.expand')">
            <h2 class="section-title">{{ $t('search.seriesResults', { count: data.series.length }) }}</h2>
            <IconChevron class="chevron-icon" :direction="isSeriesExpanded ? 'down' : 'right'" />
          </button>
          <div v-show="isSeriesExpanded" class="media-grid">
            <SeriesCard
              v-for="item in data.series"
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
  flex: 1;
  min-width: 0;
  height: 100%;
}

.search-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  padding: var(--spacing-6) var(--spacing-8);
  overflow-y: auto;
}

.view-header {
  margin-bottom: var(--spacing-6);
}

.view-title {
  font-size: 1.75rem;
  font-weight: 800;
  margin: 0 0 var(--spacing-1) 0;
}

.search-meta {
  font-size: 0.95rem;
  color: var(--color-text-muted);
  margin: 0;
}

.query-highlight {
  color: var(--color-primary);
  font-weight: 700;
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
  opacity: 0.3;
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
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: var(--spacing-4);
}

/* Metadata detail panel styling (matching MoviesView & SeriesView) */
.movie-metadata-box,
.series-metadata-box {
  padding: var(--spacing-4);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
}
</style>
