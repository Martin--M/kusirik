<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useSeriesCategories, useSeries, useSeriesInfo } from '@/composables/useSeries'
import { usePlayer } from '@/composables/usePlayer'
import SeriesList from '@/components/series/SeriesList.vue'
import CategorySidebar from '@/components/ui/CategorySidebar.vue'
import FilterHeader from '@/components/ui/FilterHeader.vue'
import StreamDetailPanel from '@/components/ui/StreamDetailPanel.vue'
import type { Series, SeriesCategory } from '@/types/series'
import { useProfileStore } from '@/stores/profile.store'
import { useToastStore } from '@/stores/toast.store'
import { getSetting, copyToSystemClipboard } from '@/lib/tauri-commands'
import { buildEpisodeUrl } from '@/lib/url-builder'

const selectedCategoryId = ref<string>('all')
const selectedSeries = ref<Series | null>(null)
const searchQuery = ref('')
const sortField = ref<'name' | 'rating'>('name')
const sortOrder = ref<'asc' | 'desc'>('asc')
const isGridView = ref(true)

const isMobileDetailOpen = ref(false)
const expandedSeason = ref<string | number | null>(null)

const sortLabels = {
  name: 'Alphabetical',
  rating: 'Rating'
}

const profileStore = useProfileStore()
const toastStore = useToastStore()

const { data: categoriesData, isLoading: isLoadingCategories } = useSeriesCategories()

// Computed categories list including "All" and "Uncategorized"
const categories = computed<SeriesCategory[]>(() => {
  const list: SeriesCategory[] = [
    { profile_id: 1, category_id: 'all', category_name: 'All TV Series' },
    { profile_id: 1, category_id: '0', category_name: 'Uncategorized' }
  ]
  if (categoriesData.value) {
    const providerCats = categoriesData.value.filter(
      (c) => c.category_id !== '0' && c.category_id !== 'all'
    )
    list.push(...providerCats)
  }
  return list
})

// Feed reactive categoryId to the series query
const seriesQueryId = computed(() => {
  if (selectedCategoryId.value === 'all') return undefined
  return selectedCategoryId.value
})

const { data: rawSeries, isLoading: isLoadingSeries } = useSeries(seriesQueryId)

// Reset selection when changing categories
watch(selectedCategoryId, () => {
  selectedSeries.value = null
  expandedSeason.value = null
})

// Filter and sort series on client side
const filteredSeriesList = computed(() => {
  let result = rawSeries.value || []

  // Apply search query
  const query = searchQuery.value.toLowerCase().trim()
  if (query) {
    result = result.filter(
      (s) => s.name && s.name.toLowerCase().includes(query)
    )
  }

  // Apply sort
  return [...result].sort((a, b) => {
    if (sortField.value === 'name') {
      const nameA = a.name || ''
      const nameB = b.name || ''
      return sortOrder.value === 'asc'
        ? nameA.localeCompare(nameB)
        : nameB.localeCompare(nameA)
    } else if (sortField.value === 'rating') {
      const ratingA = parseFloat(a.rating || '0')
      const ratingB = parseFloat(b.rating || '0')
      return sortOrder.value === 'asc' ? ratingB - ratingA : ratingA - ratingB
    }
    return 0
  })
})

// Selected series ID for on-demand details query
const selectedSeriesId = computed(() => selectedSeries.value?.series_id || 0)
const { data: seriesDetails, isLoading: isLoadingDetails } = useSeriesInfo(selectedSeriesId)

const mappedSelectedSeries = computed(() => {
  if (!selectedSeries.value) return null
  return {
    ...selectedSeries.value,
    stream_id: selectedSeries.value.series_id,
    name: selectedSeries.value.name,
    stream_icon: selectedSeries.value.cover
  }
})

// If details load, automatically expand the first season
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

const { playEpisode } = usePlayer()

function selectSeries(series: Series) {
  selectedSeries.value = series
  isMobileDetailOpen.value = true
}

function handlePlayEpisode(episode: any) {
  const ext = episode.container_extension || 'mp4'
  // Convert episode.id from string or number to number as required by usePlayer
  const episodeStreamId = typeof episode.id === 'string' ? parseInt(episode.id, 10) : episode.id
  playEpisode(episodeStreamId, ext)
}

function closeDetails() {
  selectedSeries.value = null
  isMobileDetailOpen.value = false
  expandedSeason.value = null
}

function toggleSeason(seasonKey: string | number) {
  if (expandedSeason.value === seasonKey) {
    expandedSeason.value = null
  } else {
    expandedSeason.value = seasonKey
  }
}

async function copyEpisodeUrl(episode: any) {
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
    toastStore.showToast('Episode stream URL copied to clipboard!', 'success')
  } catch (e) {
    console.error(e)
    toastStore.showToast('Failed to copy stream URL.', 'error')
  }
}
</script>

<template>
  <div class="series-view-container">
    <!-- Categories Sidebar (Desktop Only) -->
    <CategorySidebar
      layout="sidebar"
      :categories="categories"
      :selected-id="selectedCategoryId"
      :is-loading="isLoadingCategories"
      @select="selectedCategoryId = $event"
    />

    <!-- Main List/Grid & Filters Area -->
    <section class="series-section">
      <!-- Search & Filters Header -->
      <div class="filter-header">
        <!-- Categories Chip Bar (Mobile Only) -->
        <CategorySidebar
          layout="chips"
          :categories="categories"
          :selected-id="selectedCategoryId"
          :is-loading="isLoadingCategories"
          @select="selectedCategoryId = $event"
        />

        <FilterHeader
          v-model:search-query="searchQuery"
          v-model:sort-field="sortField"
          v-model:sort-order="sortOrder"
          v-model:is-grid-view="isGridView"
          :sort-labels="sortLabels"
          search-placeholder="Search series by name..."
        />
      </div>

      <!-- Virtualized Scroll List / Grid -->
      <div class="list-wrapper">
        <div v-if="isLoadingSeries" class="loading-series">
          <div v-for="i in 8" :key="i" class="skeleton-grid-card">
            <div class="skeleton-poster"></div>
            <div class="skeleton-text"></div>
          </div>
        </div>
        <SeriesList
          v-else
          :series-list="filteredSeriesList"
          :selected-series-id="selectedSeries?.series_id || null"
          :is-grid-view="isGridView"
          @select="selectSeries"
          @play="selectSeries"
        />
      </div>
    </section>

    <!-- Details Sidebar (Desktop Only) & Bottom Sheet (Mobile Only) -->
    <StreamDetailPanel
      :stream="mappedSelectedSeries"
      :is-mobile-open="isMobileDetailOpen"
      :show-actions="false"
      @close="closeDetails"
    >
      <template #header-meta-mobile>
        <span v-if="selectedSeries?.rating && parseFloat(selectedSeries.rating) > 0" class="rating-text-chip">
          <svg viewBox="0 0 24 24" fill="currentColor" style="width: 12px; height: 12px; display: inline-block; vertical-align: -1px; margin-right: 4px;">
            <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" />
          </svg>
          <span>{{ parseFloat(selectedSeries.rating).toFixed(1) }}</span>
        </span>
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

          <!-- Seasons Accordion Tree -->
          <div class="seasons-accordion" v-if="seriesDetails.episodes && Object.keys(seriesDetails.episodes).length > 0">
            <h4 class="section-subtitle accordion-heading">Seasons & Episodes</h4>
            
            <div
              v-for="(episodesList, seasonKey) in seriesDetails.episodes"
              :key="seasonKey"
              class="season-group"
              :class="{ expanded: expandedSeason === seasonKey }"
            >
              <!-- Season Header Trigger -->
              <button class="season-header" @click="toggleSeason(seasonKey)">
                <span class="season-title">Season {{ seasonKey }}</span>
                <span class="episode-count">{{ episodesList.length }} Episodes</span>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" class="chevron-icon">
                  <path d="M6 9l6 6 6-6" />
                </svg>
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
                      {{ episode.title || `Episode ${episode.episode_num}` }}
                    </span>
                  </div>
                  <div class="episode-info" v-if="episode.info?.plot">
                    <p class="episode-plot">{{ episode.info.plot }}</p>
                  </div>
                  <div class="episode-actions">
                    <button class="action-btn play-btn" @click="handlePlayEpisode(episode)">
                      <svg viewBox="0 0 24 24" fill="currentColor" class="action-icon">
                        <path d="M8 5v14l11-7z" />
                      </svg>
                      <span>Play</span>
                    </button>
                    <button class="action-btn copy-btn" @click="copyEpisodeUrl(episode)" title="Copy URL">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="action-icon">
                        <path d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div v-else class="no-episodes-msg">
            <p>No episodes listings available for this series.</p>
          </div>
        </div>
        <div v-else class="metadata-empty">
          <p>No details found for this series.</p>
        </div>
      </div>

      <template #no-selection>
        <div class="no-selection">
          <div class="tv-art">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
              <rect x="2" y="7" width="20" height="15" rx="2" ry="2" />
              <path d="M17 2l-5 5-5-5" stroke-linecap="round" stroke-linejoin="round" />
              <path d="M8 15h8M12 11v8" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </div>
          <p>Select a TV series to load seasons, episodes, and descriptions.</p>
        </div>
      </template>
    </StreamDetailPanel>
  </div>
</template>

<style scoped>
.series-view-container {
  display: flex;
  height: 100%;
  width: 100%;
  background-color: var(--color-bg);
  min-height: 0;
}

.series-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  height: 100%;
  border-right: 1px solid var(--color-border);
}

.filter-header {
  padding: var(--spacing-4);
  background-color: rgba(15, 23, 42, 0.2);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

[data-theme='light'] .filter-header {
  background-color: rgba(255, 255, 255, 0.2);
}

.list-wrapper {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.loading-series {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: var(--spacing-4);
  padding: var(--spacing-4);
  overflow: hidden;
}

.skeleton-grid-card {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
}

.skeleton-poster {
  aspect-ratio: 2/3;
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-md);
  animation: pulse 1.5s infinite ease-in-out;
}

.skeleton-text {
  height: 14px;
  width: 70%;
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-sm);
  animation: pulse 1.5s infinite ease-in-out;
}

@keyframes pulse {
  0% { opacity: 0.6; }
  50% { opacity: 1; }
  100% { opacity: 0.6; }
}

.series-metadata-box {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.metadata-loading {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
  padding: var(--spacing-2);
}

.skeleton-meta-line {
  height: 12px;
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-sm);
  animation: pulse 1.5s infinite;
}

.skeleton-meta-line.short {
  width: 50%;
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
  color: var(--color-text-muted);
  line-height: 1.5;
  font-size: 0.825rem;
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
