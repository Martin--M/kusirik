<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useVodCategories, useVodStreams, useVodInfo } from '@/composables/useVodStreams'
import { usePlayer } from '@/composables/usePlayer'
import MovieList from '@/components/movies/MovieList.vue'
import CategorySidebar from '@/components/ui/CategorySidebar.vue'
import FilterHeader from '@/components/ui/FilterHeader.vue'
import StreamDetailPanel from '@/components/ui/StreamDetailPanel.vue'
import type { VodStream, VodCategory } from '@/types/vod'
import { useProfileStore } from '@/stores/profile.store'
import { useToastStore } from '@/stores/toast.store'
import { useI18n } from '@/composables/useI18n'
import { getSetting, copyToSystemClipboard } from '@/lib/tauri-commands'
import { buildMovieUrl } from '@/lib/url-builder'
import IconStar from '@/components/icons/IconStar.vue'
import IconTVGrid from '@/components/icons/IconTVGrid.vue'

const selectedCategoryId = ref<string>('all')
const selectedStream = ref<VodStream | null>(null)
const searchQuery = ref('')
const sortField = ref<'name' | 'rating' | 'added'>('name')
const sortOrder = ref<'asc' | 'desc'>('asc')
const isGridView = ref(true)

const isMobileDetailOpen = ref(false)

const { t } = useI18n()

const sortLabels = computed(() => ({
  name: t('sortField.name'),
  rating: t('sortField.rating'),
  added: t('sortField.added')
}))

const profileStore = useProfileStore()
const toastStore = useToastStore()

const { data: categoriesData, isLoading: isLoadingCategories } = useVodCategories()

// Computed categories list including "All" and "Uncategorized"
const categories = computed<VodCategory[]>(() => {
  const list: VodCategory[] = [
    { profile_id: 1, category_id: 'all', category_name: t('media.allMovies') },
    { profile_id: 1, category_id: '0', category_name: t('media.uncategorized') }
  ]
  if (categoriesData.value) {
    const providerCats = categoriesData.value.filter(
      (c) => c.category_id !== '0' && c.category_id !== 'all'
    )
    list.push(...providerCats)
  }
  return list
})


// Feed reactive categoryId to the streams query
const streamsQueryId = computed(() => {
  if (selectedCategoryId.value === 'all') return undefined
  return selectedCategoryId.value
})

const { data: rawStreams, isLoading: isLoadingStreams } = useVodStreams(streamsQueryId)

// Reset selection when changing categories
watch(selectedCategoryId, () => {
  selectedStream.value = null
})

// Filter and sort streams on client side
const filteredStreams = computed(() => {
  let result = rawStreams.value || []

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
      // asc: highest (10) -> lowest (0); desc: lowest (0) -> highest (10)
      return sortOrder.value === 'asc' ? ratingB - ratingA : ratingA - ratingB
    } else if (sortField.value === 'added') {
      const addedA = parseInt(a.added || '0', 10)
      const addedB = parseInt(b.added || '0', 10)
      // asc: recent -> least recent; desc: least recent -> recent
      return sortOrder.value === 'asc' ? addedB - addedA : addedA - addedB
    }
    return 0
  })
})

// Selected stream ID for on-demand details query
const selectedStreamId = computed(() => selectedStream.value?.stream_id || 0)
const { data: movieInfo, isLoading: isLoadingInfo } = useVodInfo(selectedStreamId)

const { playMovie } = usePlayer()

function selectMovie(movie: VodStream) {
  selectedStream.value = movie
  isMobileDetailOpen.value = true
}

function handlePlay(movie: VodStream) {
  playMovie(movie.stream_id, movie.container_extension || 'mp4')
}

function closeDetails() {
  selectedStream.value = null
  isMobileDetailOpen.value = false
}

async function copyUrl(movie: VodStream) {
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

    const ext = movie.container_extension || 'mp4'
    const url = buildMovieUrl(
      {
        serverUrl: profile.server_url,
        username: profile.username,
        password,
      },
      movie.stream_id,
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
  <div class="movies-view-container">
    <!-- Categories Sidebar (Desktop Only) -->
    <CategorySidebar
      layout="sidebar"
      :categories="categories"
      :selected-id="selectedCategoryId"
      :is-loading="isLoadingCategories"
      @select="selectedCategoryId = $event"
    />

    <!-- Main List/Grid & Filters Area -->
    <section class="movies-section">
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
          show-layout-toggle
          :search-placeholder="$t('media.searchMovies')"
        />
      </div>

      <!-- Virtualized Scroll List / Grid -->
      <div class="list-wrapper">
        <div v-if="isLoadingStreams" class="loading-streams">
          <div v-for="i in 8" :key="i" class="skeleton-grid-card">
            <div class="skeleton-poster"></div>
            <div class="skeleton-text"></div>
          </div>
        </div>
        <MovieList
          v-else
          :streams="filteredStreams"
          :selected-stream-id="selectedStream?.stream_id || null"
          :is-grid-view="isGridView"
          @select="selectMovie"
          @play="handlePlay"
        />
      </div>
    </section>

    <!-- Details Sidebar (Desktop Only) & Bottom Sheet (Mobile Only) -->
    <StreamDetailPanel
      :stream="selectedStream"
      :is-mobile-open="isMobileDetailOpen"
      :play-button-text="$t('media.play')"
      :copy-button-text="$t('media.copyUrl')"
      @close="closeDetails"
      @play="handlePlay(selectedStream!)"
      @copy="copyUrl(selectedStream!)"
    >
      <template #header-meta-mobile>
        <span v-if="selectedStream?.rating && parseFloat(selectedStream.rating) > 0" class="rating-text-chip">
          <IconStar style="width: 12px; height: 12px; display: inline-block; vertical-align: -1px; margin-right: 4px;" />
          <span>{{ parseFloat(selectedStream.rating).toFixed(1) }}</span>
        </span>
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
  </div>
</template>

<style scoped>
.movies-view-container {
  display: flex;
  height: 100%;
  width: 100%;
  background-color: var(--color-bg);
  min-height: 0;
}

.movies-section {
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

.loading-streams {
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

.movie-metadata-box {
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
</style>
