<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useGlobalSearch } from '@/composables/useSearch'
import { usePlayer } from '@/composables/usePlayer'
import { useVodInfo } from '@/composables/useVodStreams'
import { useSeriesInfo } from '@/composables/useSeries'
import { useToastStore } from '@/stores/toast.store'
import { useProfileStore } from '@/stores/profile.store'
import { getSetting } from '@/lib/tauri-commands'

import ChannelRow from '@/components/live/ChannelRow.vue'
import MovieCard from '@/components/movies/MovieCard.vue'
import SeriesCard from '@/components/series/SeriesCard.vue'
import StreamDetailPanel from '@/components/ui/StreamDetailPanel.vue'

import IconStar from '@/components/icons/IconStar.vue'
import IconChevron from '@/components/icons/IconChevron.vue'
import IconPlay from '@/components/icons/IconPlay.vue'
import IconTVGrid from '@/components/icons/IconTVGrid.vue'
import IconSearch from '@/components/icons/IconSearch.vue'

import type { LiveStream } from '@/types/stream'
import type { VodStream } from '@/types/vod'
import type { Series } from '@/types/series'

const route = useRoute()
const toastStore = useToastStore()
const profileStore = useProfileStore()
const { playLive, playMovie, playEpisode } = usePlayer()

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

// Series accordion state
const expandedSeason = ref<string | null>(null)

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
  expandedSeason.value = null
}

// Fetch detail metadata for active VOD / Series selections
const selectedMovieId = computed(() => selectedMovie.value?.stream_id || 0)
const { data: movieInfo, isLoading: isLoadingMovieInfo } = useVodInfo(selectedMovieId)

const selectedSeriesId = computed(() => selectedSeries.value?.series_id || 0)
const { data: seriesDetails, isLoading: isLoadingSeriesInfo } = useSeriesInfo(selectedSeriesId)

// Map series structure for the shared StreamDetailPanel
const mappedSelectedSeries = computed(() => {
  if (!selectedSeries.value) return null
  return {
    stream_id: selectedSeries.value.series_id,
    name: selectedSeries.value.name || '',
    stream_icon: selectedSeries.value.cover || '',
    rating: selectedSeries.value.rating || '',
  }
})

// Play handles
function handlePlayLive(stream: LiveStream) {
  playLive(stream.stream_id)
}

function handlePlayMovie(stream: VodStream) {
  playMovie(stream.stream_id, stream.container_extension || 'mp4')
}

function toggleSeason(seasonKey: string) {
  expandedSeason.value = expandedSeason.value === seasonKey ? null : seasonKey
}

function handlePlayEpisode(episode: any) {
  playEpisode(episode.id, episode.container_extension || 'mp4')
}

// Copy URLs fallback
async function copyUrl(stream: any, type: 'live' | 'movie') {
  try {
    const profile = profileStore.profile
    if (!profile) {
      toastStore.showToast('No active profile loaded.', 'error')
      return
    }

    const password = await getSetting('password')
    if (!password) {
      toastStore.showToast('Could not retrieve credentials.', 'error')
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
    toastStore.showToast('Stream URL copied to clipboard!', 'success')
  } catch (e: any) {
    toastStore.showToast(e?.message || e || 'Failed to copy URL.', 'error')
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
        <h1 class="view-title">Search Results</h1>
        <p v-if="queryText" class="search-meta">
          Showing results for <span class="query-highlight">"{{ queryText }}"</span>
        </p>
      </header>

      <!-- Empty / Idle State -->
      <div v-if="!queryText || queryText.length < 2" class="search-empty-state">
        <IconSearch class="empty-icon" />
        <h3>Type at least 2 characters to search...</h3>
        <p>You can search for live channels, VOD movies, and TV series simultaneously.</p>
      </div>

      <!-- Loading State -->
      <div v-else-if="isLoading" class="search-loading-state">
        <span class="spinner"></span>
        <p>Searching lists...</p>
      </div>

      <!-- No Results State -->
      <div v-else-if="data && !data.live.length && !data.vod.length && !data.series.length" class="search-empty-state">
        <IconSearch class="empty-icon" />
        <h3>No results found for "{{ queryText }}"</h3>
        <p>Try searching with different keywords.</p>
      </div>

      <!-- Results List -->
      <div v-else class="results-layout">
        <!-- Live TV Section -->
        <section v-if="data?.live.length" class="results-section" :class="{ collapsed: !isLiveExpanded }">
          <button class="section-toggle-btn" @click="isLiveExpanded = !isLiveExpanded" :title="isLiveExpanded ? 'Collapse' : 'Expand'">
            <h2 class="section-title">Live Channels ({{ data.live.length }})</h2>
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
          <button class="section-toggle-btn" @click="isMoviesExpanded = !isMoviesExpanded" :title="isMoviesExpanded ? 'Collapse' : 'Expand'">
            <h2 class="section-title">VOD Movies ({{ data.vod.length }})</h2>
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
          <button class="section-toggle-btn" @click="isSeriesExpanded = !isSeriesExpanded" :title="isSeriesExpanded ? 'Collapse' : 'Expand'">
            <h2 class="section-title">TV Series ({{ data.series.length }})</h2>
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
    <StreamDetailPanel
      v-if="selectedLive"
      :stream="selectedLive"
      :is-mobile-open="isMobileDetailOpen"
      play-button-text="Play Live Channel"
      copy-button-text="Copy Channel Stream URL"
      @close="closeDetails"
      @play="handlePlayLive(selectedLive)"
      @copy="copyUrl(selectedLive, 'live')"
    >
      <div class="live-info-box">
        <p>Click Play to stream this channel live in your configured media player.</p>
      </div>
      <template #no-selection>
        <div></div>
      </template>
    </StreamDetailPanel>

    <!-- Movie Details Panel -->
    <StreamDetailPanel
      v-if="selectedMovie"
      :stream="selectedMovie"
      :is-mobile-open="isMobileDetailOpen"
      play-button-text="Play Movie"
      copy-button-text="Copy Movie Stream URL"
      @close="closeDetails"
      @play="handlePlayMovie(selectedMovie)"
      @copy="copyUrl(selectedMovie, 'movie')"
    >
      <div class="movie-metadata-box">
        <div v-if="isLoadingMovieInfo" class="metadata-loading">
          <div class="skeleton-meta-line"></div>
          <div class="skeleton-meta-line short"></div>
          <div class="skeleton-meta-line"></div>
        </div>
        <div v-else-if="movieInfo" class="metadata-content">
          <div class="meta-row" v-if="movieInfo.info?.releasedate">
            <span class="meta-label">Released:</span>
            <span class="meta-value">{{ movieInfo.info.releasedate }}</span>
          </div>
          <div class="meta-row" v-if="movieInfo.info?.duration">
            <span class="meta-label">Duration:</span>
            <span class="meta-value">{{ movieInfo.info.duration }}</span>
          </div>
          <div class="meta-row" v-if="movieInfo.info?.genre">
            <span class="meta-label">Genre:</span>
            <span class="meta-value">{{ movieInfo.info.genre }}</span>
          </div>
          <div class="meta-row" v-if="movieInfo.info?.director">
            <span class="meta-label">Director:</span>
            <span class="meta-value">{{ movieInfo.info.director }}</span>
          </div>
          <div class="meta-row" v-if="movieInfo.info?.cast || movieInfo.info?.actors">
            <span class="meta-label">Cast:</span>
            <span class="meta-value text-clamp" :title="movieInfo.info.cast || movieInfo.info.actors">
              {{ movieInfo.info.cast || movieInfo.info.actors }}
            </span>
          </div>
          <div class="meta-plot" v-if="movieInfo.info?.plot || movieInfo.info?.description">
            <h4 class="section-subtitle">Synopsis</h4>
            <p class="plot-text">{{ movieInfo.info.plot || movieInfo.info.description }}</p>
          </div>
        </div>
        <div v-else class="metadata-empty">
          <p>No additional details loaded for this movie.</p>
        </div>
      </div>
      <template #no-selection>
        <div></div>
      </template>
    </StreamDetailPanel>

    <!-- TV Series Details Panel -->
    <StreamDetailPanel
      v-if="selectedSeries"
      :stream="mappedSelectedSeries"
      :is-mobile-open="isMobileDetailOpen"
      :show-actions="false"
      @close="closeDetails"
    >
      <div class="series-metadata-box">
        <div v-if="isLoadingSeriesInfo" class="metadata-loading">
          <div class="skeleton-meta-line"></div>
          <div class="skeleton-meta-line short"></div>
          <div class="skeleton-meta-line"></div>
        </div>
        <div v-else-if="seriesDetails" class="metadata-content">
          <div class="meta-row" v-if="selectedSeries?.genre || seriesDetails.info?.genre">
            <span class="meta-label">Genre:</span>
            <span class="meta-value">{{ selectedSeries?.genre || seriesDetails.info?.genre }}</span>
          </div>
          <div class="meta-row" v-if="selectedSeries?.director || seriesDetails.info?.director">
            <span class="meta-label">Director:</span>
            <span class="meta-value">{{ selectedSeries?.director || seriesDetails.info?.director }}</span>
          </div>
          <div class="meta-row" v-if="selectedSeries?.cast_ || seriesDetails.info?.cast || seriesDetails.info?.actors">
            <span class="meta-label">Cast:</span>
            <span class="meta-value text-clamp" :title="selectedSeries?.cast_ || seriesDetails.info?.cast || seriesDetails.info?.actors">
              {{ selectedSeries?.cast_ || seriesDetails.info?.cast || seriesDetails.info?.actors }}
            </span>
          </div>
          <div class="meta-plot" v-if="selectedSeries?.plot || seriesDetails.info?.plot || seriesDetails.info?.description">
            <h4 class="section-subtitle">Synopsis</h4>
            <p class="plot-text">{{ selectedSeries?.plot || seriesDetails.info?.plot || seriesDetails.info?.description }}</p>
          </div>

          <div class="seasons-accordion" v-if="seriesDetails.episodes && Object.keys(seriesDetails.episodes).length > 0">
            <h4 class="section-subtitle accordion-heading">Seasons & Episodes</h4>
            
            <div
              v-for="(episodesList, seasonKey) in seriesDetails.episodes"
              :key="seasonKey"
              class="season-group"
              :class="{ expanded: expandedSeason === seasonKey }"
            >
              <button class="season-header" @click="toggleSeason(seasonKey)">
                <span class="season-title">Season {{ seasonKey }}</span>
                <span class="episode-count">{{ episodesList.length }} Episodes</span>
                <IconChevron class="chevron-icon" />
              </button>

              <div class="episodes-container" v-show="expandedSeason === seasonKey">
                <div
                  v-for="episode in episodesList"
                  :key="episode.id"
                  class="episode-row"
                >
                  <div class="episode-main">
                    <span class="episode-num">Ep {{ episode.episode_num }}</span>
                    <span class="episode-title" :title="episode.title || ''">
                      {{ episode.title || `Episode ${episode.episode_num}` }}
                    </span>
                  </div>
                  <div class="episode-info" v-if="episode.info?.plot">
                    <p class="episode-plot">{{ episode.info.plot }}</p>
                  </div>
                  <div class="episode-actions">
                    <button class="action-btn play-btn" @click="handlePlayEpisode(episode)" title="Play Episode">
                      <IconPlay class="action-icon" />
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div v-else class="metadata-empty">
          <p>No additional details loaded for this series.</p>
        </div>
      </div>
      <template #no-selection>
        <div></div>
      </template>
    </StreamDetailPanel>
  </div>
</template>

<style scoped>
.search-view-container {
  display: flex;
  flex: 1;
  min-width: 0;
  height: 100vh;
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

.metadata-loading {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.skeleton-meta-line {
  height: 12px;
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: 9999px;
  width: 100%;
}

.skeleton-meta-line.short {
  width: 60%;
}

.metadata-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.meta-row {
  display: flex;
  font-size: 0.85rem;
  line-height: 1.4;
}

.meta-label {
  color: var(--color-text-muted);
  width: 80px;
  flex-shrink: 0;
  font-weight: 500;
}

.meta-value {
  color: var(--color-text);
}

.text-clamp {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.meta-plot {
  margin-top: var(--spacing-2);
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
