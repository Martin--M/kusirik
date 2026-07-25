<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useVodCategories, useVodStreams } from '@/composables/useVodStreams'
import { usePlayer } from '@/composables/usePlayer'
import MovieList from '@/components/movies/MovieList.vue'
import CategorySidebar from '@/components/ui/CategorySidebar.vue'
import FilterHeader from '@/components/ui/FilterHeader.vue'
import MovieDetailPanel from '@/components/movies/MovieDetailPanel.vue'
import type { VodStream, VodCategory } from '@/types/vod'
import { useProfileStore } from '@/stores/profile.store'
import { useToastStore } from '@/stores/toast.store'
import { useI18n } from '@/composables/useI18n'
import { copyToSystemClipboard } from '@/lib/tauri-commands'
import { buildMovieUrl } from '@/lib/url-builder'
import { useToggleFavorite } from '@/composables/useFavorites'

const selectedCategoryId = ref<string>('all')
const selectedStream = ref<VodStream | null>(null)
const { toggle: toggleFav } = useToggleFavorite()

async function handleToggleFavorite(stream: VodStream) {
  const originalState = selectedStream.value?.is_favorite
  const nextState = originalState === 1 ? 0 : 1

  // Optimistic update
  if (selectedStream.value && selectedStream.value.stream_id === stream.stream_id) {
    selectedStream.value = {
      ...selectedStream.value,
      is_favorite: nextState
    }
  }

  try {
    const isFav = await toggleFav('vod', stream.stream_id)
    if (selectedStream.value && selectedStream.value.stream_id === stream.stream_id) {
      selectedStream.value = {
        ...selectedStream.value,
        is_favorite: isFav ? 1 : 0
      }
    }
  } catch (err) {
    console.error('Failed to toggle favorite:', err)
    // Revert on error
    if (selectedStream.value && selectedStream.value.stream_id === stream.stream_id) {
      selectedStream.value = {
        ...selectedStream.value,
        is_favorite: originalState
      }
    }
    toastStore.showToast(t('media.favoriteToggleFailed') || 'Failed to update favorite status', 'error')
  }
}
const searchQuery = ref('')
const sortField = ref<'name' | 'rating' | 'added' | 'releaseDate'>('name')
const sortOrder = ref<'asc' | 'desc'>('asc')
const isGridView = ref(true)

const isMobileDetailOpen = ref(false)

const { t } = useI18n()

const sortLabels = computed(() => ({
  name: t('sortField.name'),
  rating: t('sortField.rating'),
  added: t('sortField.added'),
  releaseDate: t('sortField.releaseDate')
}))

const profileStore = useProfileStore()
const toastStore = useToastStore()

const { data: categoriesData, isLoading: isLoadingCategories } = useVodCategories(
  computed(() => profileStore.filterProfileId)
)

// Computed categories list including "All" and "Uncategorized"
const categories = computed<VodCategory[]>(() => {
  const currentProfileId = profileStore.profile?.id
  if (currentProfileId === undefined) {
    return []
  }
  const list: VodCategory[] = [
    { profile_id: currentProfileId, category_id: 'all', category_name: t('media.allMovies') },
    { profile_id: currentProfileId, category_id: '0', category_name: t('media.uncategorized') }
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

const { data: rawStreams, isLoading: isLoadingStreams } = useVodStreams(
  streamsQueryId,
  computed(() => profileStore.filterProfileId)
)

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
      const addedA = a.added || 0
      const addedB = b.added || 0
      // asc: recent -> least recent; desc: least recent -> recent
      return sortOrder.value === 'asc' ? addedB - addedA : addedA - addedB
    } else if (sortField.value === 'releaseDate') {
      const tsA = a.release_date || 0
      const tsB = b.release_date || 0
      if (tsA === 0 && tsB === 0) return 0
      if (tsA === 0) return 1
      if (tsB === 0) return -1
      // asc: newest -> oldest; desc: oldest -> newest
      return sortOrder.value === 'asc' ? tsB - tsA : tsA - tsB
    }
    return 0
  })
})



const { playMovie } = usePlayer()

function selectMovie(movie: VodStream) {
  selectedStream.value = movie
  isMobileDetailOpen.value = true
}

function handlePlay(movie: VodStream) {
  playMovie(movie.stream_id, movie.container_extension || 'mp4', movie.profile_id)
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

    const password = profile.password
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
    <MovieDetailPanel
      :stream="selectedStream"
      :is-mobile-open="isMobileDetailOpen"
      @close="closeDetails"
      @play="handlePlay"
      @copy="copyUrl"
      @toggle-favorite="handleToggleFavorite"
    />
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

</style>
