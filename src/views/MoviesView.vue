<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { useVodCategories, useVodStreams, useVodInfo } from '@/composables/useVodStreams'
import { usePlayer } from '@/composables/usePlayer'
import MovieList from '@/components/movies/MovieList.vue'
import CachedImage from '@/components/ui/CachedImage.vue'
import type { VodStream, VodCategory } from '@/types/vod'
import { useProfileStore } from '@/stores/profile.store'
import { getSetting, copyToSystemClipboard } from '@/lib/tauri-commands'
import { buildMovieUrl } from '@/lib/url-builder'

const selectedCategoryId = ref<string>('all')
const selectedStream = ref<VodStream | null>(null)
const searchQuery = ref('')
const sortField = ref<'name' | 'rating' | 'added'>('name')
const sortOrder = ref<'asc' | 'desc'>('asc')
const isGridView = ref(true)

const isSortDropdownOpen = ref(false)
const sortLabels = {
  name: 'Alphabetical',
  rating: 'Rating',
  added: 'Date Added'
}

function selectSortField(field: 'name' | 'rating' | 'added') {
  sortField.value = field
  isSortDropdownOpen.value = false
}

function closeDropdown(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.custom-dropdown')) {
    isSortDropdownOpen.value = false
  }
}

onMounted(() => {
  window.addEventListener('click', closeDropdown)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdown)
})

const isMobileDetailOpen = ref(false)

const profileStore = useProfileStore()

const toast = ref({
  show: false,
  message: '',
  type: 'success' as 'success' | 'error'
})

let toastTimeout: number | null = null

function showToast(message: string, type: 'success' | 'error' = 'success') {
  toast.value.message = message
  toast.value.type = type
  toast.value.show = true

  if (toastTimeout) {
    clearTimeout(toastTimeout)
  }

  toastTimeout = window.setTimeout(() => {
    toast.value.show = false
  }, 3000)
}

const { data: categoriesData, isLoading: isLoadingCategories } = useVodCategories()

// Computed categories list including "All" and "Uncategorized"
const categories = computed<VodCategory[]>(() => {
  const list: VodCategory[] = [
    { profile_id: 1, category_id: 'all', category_name: 'All Movies' },
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

function toggleSort() {
  sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
}

async function copyUrl(movie: VodStream) {
  try {
    const profile = profileStore.profile
    if (!profile) {
      showToast('No active profile loaded.', 'error')
      return
    }

    const password = await getSetting('password')
    if (!password) {
      showToast('Could not retrieve credentials.', 'error')
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
    showToast('Movie stream URL copied to clipboard!', 'success')
  } catch (e) {
    console.error(e)
    showToast('Failed to copy stream URL.', 'error')
  }
}
</script>

<template>
  <div class="movies-view-container">
    <!-- Categories Sidebar (Desktop Only) -->
    <aside class="categories-sidebar desktop-only">
      <div v-if="isLoadingCategories" class="loading-sidebar">
        <div v-for="i in 8" :key="i" class="skeleton-pill"></div>
      </div>
      <div v-else class="categories-list">
        <button
          v-for="cat in categories"
          :key="cat.category_id"
          class="category-btn"
          :class="{ active: selectedCategoryId === cat.category_id }"
          @click="selectedCategoryId = cat.category_id"
        >
          <span class="category-name">{{ cat.category_name }}</span>
        </button>
      </div>
    </aside>

    <!-- Main List/Grid & Filters Area -->
    <section class="movies-section">
      <!-- Search & Filters Header -->
      <div class="filter-header">
        <!-- Categories Chip Bar (Mobile Only) -->
        <div class="mobile-categories mobile-only">
          <button
            v-for="cat in categories"
            :key="cat.category_id"
            class="chip-btn"
            :class="{ active: selectedCategoryId === cat.category_id }"
            @click="selectedCategoryId = cat.category_id"
          >
            {{ cat.category_name }}
          </button>
        </div>

        <div class="search-sort-bar">
          <div class="search-wrapper">
            <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            <input
              v-model="searchQuery"
              type="text"
              placeholder="Search movies by name..."
              class="search-input"
            />
            <button v-if="searchQuery" class="clear-search" @click="searchQuery = ''">×</button>
          </div>

          <div class="sort-controls">
            <!-- Grid/List Toggle Button -->
            <button
              class="layout-toggle-btn"
              @click="isGridView = !isGridView"
              :title="isGridView ? 'Switch to List View' : 'Switch to Grid View'"
            >
              <svg v-if="isGridView" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="layout-icon">
                <line x1="4" y1="6" x2="20" y2="6" />
                <line x1="4" y1="12" x2="20" y2="12" />
                <line x1="4" y1="18" x2="20" y2="18" />
              </svg>
              <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="layout-icon">
                <rect x="3" y="3" width="7" height="7" />
                <rect x="14" y="3" width="7" height="7" />
                <rect x="14" y="14" width="7" height="7" />
                <rect x="3" y="14" width="7" height="7" />
              </svg>
            </button>

            <!-- Custom Dropdown Sort Select -->
            <div class="custom-dropdown">
              <button
                class="dropdown-trigger"
                @click.stop="isSortDropdownOpen = !isSortDropdownOpen"
                aria-haspopup="listbox"
                :aria-expanded="isSortDropdownOpen"
                title="Select Sort Field"
              >
                <span>{{ sortLabels[sortField] }}</span>
                <svg class="dropdown-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="6 9 12 15 18 9" />
                </svg>
              </button>
              <transition name="dropdown-fade">
                <ul v-if="isSortDropdownOpen" class="dropdown-menu">
                  <li
                    v-for="(label, value) in sortLabels"
                    :key="value"
                    class="dropdown-item"
                    :class="{ active: sortField === value }"
                    @click="selectSortField(value)"
                  >
                    {{ label }}
                  </li>
                </ul>
              </transition>
            </div>

            <button
              class="sort-toggle-btn"
              @click="toggleSort"
              :title="`Sort Direction: ${sortOrder === 'asc' ? 'Ascending' : 'Descending'}`"
            >
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                class="sort-icon"
                :class="{ reversed: sortOrder === 'desc' }"
              >
                <line x1="12" y1="5" x2="12" y2="19" />
                <polyline points="19 12 12 19 5 12" />
              </svg>
            </button>
          </div>
        </div>
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

    <!-- Details Sidebar (Desktop Only) -->
    <aside class="details-sidebar desktop-only">
      <transition name="fade" mode="out-in">
        <div v-if="selectedStream" :key="selectedStream.stream_id" class="details-panel">
          <button class="close-details-btn" @click="selectedStream = null" title="Close Details">×</button>
          
          <div class="details-header">
            <div class="details-poster">
              <CachedImage
                :src="selectedStream.stream_icon"
                :alt="selectedStream.name || 'Movie Cover'"
                :fallback-text="selectedStream.name || ''"
              />
            </div>
            <h3 class="movie-name-title">{{ selectedStream.name }}</h3>
          </div>

          <!-- Metadata info loading / loaded -->
          <div class="movie-metadata-box">
            <div v-if="isLoadingInfo" class="metadata-loading">
              <div class="skeleton-meta-line"></div>
              <div class="skeleton-meta-line short"></div>
              <div class="skeleton-meta-line"></div>
            </div>
            <div v-else-if="movieInfo" class="metadata-content">
              <div class="meta-row" v-if="movieInfo.info?.releasedate">
                <span class="meta-label">Released:</span>
                <span class="meta-value">{{ movieInfo.info.releasedate }}</span>
              </div>
              <div class="meta-row" v-if="movieInfo.info?.duration || movieInfo.info?.episode_run_time">
                <span class="meta-label">Duration:</span>
                <span class="meta-value">
                  {{ movieInfo.info.duration || `${movieInfo.info.episode_run_time} mins` }}
                </span>
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

          <div class="action-buttons">
            <button class="btn btn-primary" @click="handlePlay(selectedStream)">
              <svg viewBox="0 0 24 24" fill="currentColor" class="btn-icon">
                <polygon points="5 3 19 12 5 21 5 3" />
              </svg>
              Play Movie
            </button>
            <button class="btn btn-secondary" @click="copyUrl(selectedStream)">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="btn-icon">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
              </svg>
              Copy Stream URL
            </button>
          </div>
        </div>
        <div v-else class="no-selection">
          <div class="tv-art">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
              <rect x="2" y="4" width="20" height="16" rx="2" ry="2" />
              <path d="M12 17h.01M17 17h.01M7 17h.01 M12 12h.01M17 12h.01M7 12h.01 M12 7h.01M17 7h.01M7 7h.01" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </div>
          <p>Select a movie to load descriptions and start playback.</p>
        </div>
      </transition>
    </aside>

    <!-- Details Bottom Sheet (Mobile Only) -->
    <transition name="slide-up">
      <div v-if="selectedStream && isMobileDetailOpen" class="bottom-sheet-backdrop mobile-only" @click="isMobileDetailOpen = false">
        <div class="bottom-sheet-content" @click.stop>
          <div class="drag-handle"></div>
          <button class="close-sheet" @click="isMobileDetailOpen = false">×</button>
          
          <div class="details-header mobile">
            <div class="details-poster mobile">
              <CachedImage
                :src="selectedStream.stream_icon"
                :alt="selectedStream.name || 'Movie Cover'"
                :fallback-text="selectedStream.name || ''"
              />
            </div>
            <div class="header-text">
              <h3 class="movie-name-title">{{ selectedStream.name }}</h3>
              <span v-if="selectedStream.rating" class="rating-text-chip">
                ⭐ {{ selectedStream.rating }}
              </span>
            </div>
          </div>

          <div class="movie-metadata-box mobile">
            <div v-if="isLoadingInfo" class="metadata-loading">
              <div class="skeleton-meta-line"></div>
              <div class="skeleton-meta-line short"></div>
            </div>
            <div v-else-if="movieInfo" class="metadata-content">
              <p class="plot-text-mobile" v-if="movieInfo.info?.plot || movieInfo.info?.description">
                {{ movieInfo.info.plot || movieInfo.info.description }}
              </p>
              <div class="meta-row-mobile" v-if="movieInfo.info?.genre">
                <span class="meta-label">Genre:</span>
                <span class="meta-value">{{ movieInfo.info.genre }}</span>
              </div>
            </div>
          </div>

          <div class="action-buttons mobile">
            <button class="btn btn-primary" @click="handlePlay(selectedStream); isMobileDetailOpen = false">
              Play Movie
            </button>
            <button class="btn btn-secondary" @click="copyUrl(selectedStream)">
              Copy URL
            </button>
          </div>
        </div>
      </div>
    </transition>

    <!-- Toast notification overlay -->
    <transition name="toast-fade">
      <div v-if="toast.show" class="toast-message" :class="toast.type">
        <svg v-if="toast.type === 'success'" class="toast-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="20 6 9 17 4 12" />
        </svg>
        <svg v-else class="toast-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
        <span>{{ toast.message }}</span>
      </div>
    </transition>
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

/* Sidebar styling */
.categories-sidebar {
  width: 260px;
  background-color: rgba(30, 41, 59, 0.4);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  padding: var(--spacing-4);
  flex-shrink: 0;
}

[data-theme='light'] .categories-sidebar {
  background-color: rgba(240, 240, 240, 0.4);
}

.categories-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.category-btn {
  display: flex;
  align-items: center;
  width: 100%;
  padding: var(--spacing-3) var(--spacing-4);
  background: transparent;
  border: none;
  color: var(--color-text-muted);
  border-radius: var(--radius-md);
  font-family: inherit;
  font-size: 0.9rem;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.category-btn:hover {
  color: var(--color-text);
  background-color: rgba(255, 255, 255, 0.02);
}

.category-btn.active {
  color: var(--color-primary);
  background-color: rgba(96, 165, 250, 0.08);
  font-weight: 700;
}

[data-theme='light'] .category-btn.active {
  background-color: rgba(59, 130, 246, 0.05);
}

/* Middle Section */
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

.search-sort-bar {
  display: flex;
  gap: var(--spacing-3);
  align-items: center;
}

.search-wrapper {
  position: relative;
  flex: 1;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  width: 16px;
  height: 16px;
  color: var(--color-text-muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: var(--spacing-2) var(--spacing-4) var(--spacing-2) 36px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background-color: rgba(15, 23, 42, 0.4);
  color: var(--color-text);
  font-family: inherit;
  font-size: 0.9rem;
  transition: all var(--transition-fast);
}

[data-theme='light'] .search-input {
  background-color: rgba(255, 255, 255, 0.6);
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.15);
}

.clear-search {
  position: absolute;
  right: 10px;
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 1.25rem;
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.sort-controls {
  display: flex;
  gap: var(--spacing-2);
  align-items: center;
}

.layout-toggle-btn {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background-color: rgba(15, 23, 42, 0.4);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

[data-theme='light'] .layout-toggle-btn {
  background-color: rgba(255, 255, 255, 0.6);
}

.layout-toggle-btn:hover {
  color: var(--color-text);
  background-color: rgba(255, 255, 255, 0.05);
}

.layout-icon {
  width: 18px;
  height: 18px;
}

/* Custom Dropdown Styling */
.custom-dropdown {
  position: relative;
  display: inline-block;
}

.dropdown-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-2);
  padding: var(--spacing-2) var(--spacing-3);
  min-width: 120px;
  height: 34px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background-color: rgba(15, 23, 42, 0.4);
  color: var(--color-text);
  font-family: inherit;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  outline: none;
  transition: all var(--transition-fast);
}

[data-theme='light'] .dropdown-trigger {
  background-color: rgba(255, 255, 255, 0.6);
}

.dropdown-trigger:hover {
  border-color: var(--color-primary);
  background-color: rgba(255, 255, 255, 0.05);
}

[data-theme='light'] .dropdown-trigger:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.dropdown-chevron {
  width: 14px;
  height: 14px;
  color: var(--color-text-muted);
}

.dropdown-menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  min-width: 100%;
  background-color: rgba(15, 23, 42, 0.95);
  backdrop-filter: blur(10px);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-1) 0;
  margin: 0;
  list-style: none;
  z-index: 50;
  box-shadow: var(--shadow-lg), 0 10px 30px rgba(0, 0, 0, 0.5);
  transform-origin: top;
}

[data-theme='light'] .dropdown-menu {
  background-color: rgba(255, 255, 255, 0.95);
  box-shadow: var(--shadow-lg), 0 10px 30px rgba(0, 0, 0, 0.15);
}

.dropdown-item {
  padding: var(--spacing-2) var(--spacing-4);
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.dropdown-item:hover {
  color: var(--color-text);
  background-color: rgba(255, 255, 255, 0.05);
}

[data-theme='light'] .dropdown-item:hover {
  background-color: rgba(0, 0, 0, 0.03);
}

.dropdown-item.active {
  color: var(--color-primary);
  background-color: rgba(96, 165, 250, 0.08);
  font-weight: 700;
}

[data-theme='light'] .dropdown-item.active {
  background-color: rgba(59, 130, 246, 0.05);
}

/* Transitions */
.dropdown-fade-enter-active,
.dropdown-fade-leave-active {
  transition: opacity var(--transition-fast) ease, transform var(--transition-fast) ease;
}

.dropdown-fade-enter-from,
.dropdown-fade-leave-to {
  opacity: 0;
  transform: translateY(-8px) scaleY(0.95);
}

.sort-toggle-btn {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background-color: rgba(15, 23, 42, 0.4);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

[data-theme='light'] .sort-toggle-btn {
  background-color: rgba(255, 255, 255, 0.6);
}

.sort-toggle-btn:hover {
  color: var(--color-text);
  border-color: var(--color-primary);
  background-color: rgba(255, 255, 255, 0.05);
}

[data-theme='light'] .sort-toggle-btn:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.sort-icon {
  width: 14px;
  height: 14px;
  transition: transform var(--transition-normal);
}

.sort-icon.reversed {
  transform: rotate(180deg);
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

/* Details Sidebar (Desktop) */
.details-sidebar {
  width: 320px;
  background-color: rgba(30, 41, 59, 0.4);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  flex-shrink: 0;
}

[data-theme='light'] .details-sidebar {
  background-color: rgba(240, 240, 240, 0.4);
}

.details-panel {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-6);
  gap: var(--spacing-6);
  position: relative;
}

.close-details-btn {
  position: absolute;
  top: 16px;
  right: 16px;
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 1.5rem;
  cursor: pointer;
  line-height: 1;
  padding: 0;
  transition: color var(--transition-fast);
  z-index: 10;
}

.close-details-btn:hover {
  color: var(--color-text);
}

.details-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: var(--spacing-3);
}

.details-poster {
  width: 140px;
  aspect-ratio: 2/3;
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-lg), 0 0 35px rgba(96, 165, 250, 0.15);
}

.movie-name-title {
  font-size: 1.1rem;
  color: var(--color-text);
  font-weight: 700;
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
  width: 60%;
}

.metadata-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.meta-row {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-2);
  font-size: 0.825rem;
  line-height: 1.4;
}

.meta-label {
  color: var(--color-text-muted);
  font-weight: 600;
  width: 75px;
  flex-shrink: 0;
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
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-4);
}

.section-subtitle {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 700;
}

.plot-text {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  line-height: 1.5;
  margin: 0;
}

.action-buttons {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-2);
  padding: 10px 16px;
  border-radius: var(--radius-md);
  font-family: inherit;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
  border: none;
}

.btn-primary {
  background-color: var(--color-primary);
  color: white;
}

.btn-primary:hover {
  background-color: #3b82f6;
  transform: translateY(-1px);
}

.btn-secondary {
  background-color: rgba(255, 255, 255, 0.05);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.btn-secondary:hover {
  background-color: rgba(255, 255, 255, 0.08);
}

.btn-icon {
  width: 16px;
  height: 16px;
}

.no-selection {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--spacing-6);
  color: var(--color-text-muted);
  text-align: center;
  gap: var(--spacing-4);
}

.tv-art {
  width: 64px;
  height: 64px;
  opacity: 0.3;
}

/* Mobile Categories Chips */
.mobile-categories {
  display: flex;
  gap: var(--spacing-2);
  overflow-x: auto;
  padding-bottom: 2px;
  scrollbar-width: none;
}

.mobile-categories::-webkit-scrollbar {
  display: none;
}

.chip-btn {
  padding: 6px var(--spacing-3);
  border-radius: 9999px;
  border: 1px solid var(--color-border);
  background-color: rgba(15, 23, 42, 0.3);
  color: var(--color-text-muted);
  font-family: inherit;
  font-size: 0.8rem;
  font-weight: 600;
  white-space: nowrap;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.chip-btn.active {
  background-color: var(--color-primary);
  border-color: var(--color-primary);
  color: #fff;
}

/* Mobile Bottom Sheet Drawer */
@media (max-width: 768px) {
  .bottom-sheet-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.6);
    z-index: 1000;
    display: flex;
    align-items: flex-end;
  }

  .bottom-sheet-content {
    background-color: var(--color-surface);
    width: 100%;
    border-top-left-radius: var(--radius-lg);
    border-top-right-radius: var(--radius-lg);
    padding: var(--spacing-6) var(--spacing-6) calc(var(--spacing-8) + env(safe-area-inset-bottom));
    display: flex;
    flex-direction: column;
    gap: var(--spacing-5);
    position: relative;
    box-shadow: 0 -8px 24px rgba(0, 0, 0, 0.4);
    max-height: 80vh;
    overflow-y: auto;
  }

  .drag-handle {
    width: 40px;
    height: 4px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 9999px;
    align-self: center;
    margin-bottom: -4px;
  }

  .close-sheet {
    position: absolute;
    top: 16px;
    right: 16px;
    background: none;
    border: none;
    color: var(--color-text-muted);
    font-size: 1.5rem;
    cursor: pointer;
    line-height: 1;
    padding: 0;
  }

  .details-header.mobile {
    display: flex;
    flex-direction: row;
    align-items: center;
    text-align: left;
    gap: var(--spacing-4);
  }

  .details-poster.mobile {
    width: 64px;
    aspect-ratio: 2/3;
    border-radius: var(--radius-md);
  }

  .header-text {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .rating-text-chip {
    font-size: 0.75rem;
    color: #fbbf24;
    font-weight: 600;
    align-self: flex-start;
  }

  .action-buttons.mobile {
    flex-direction: row;
    gap: var(--spacing-3);
  }

  .action-buttons.mobile .btn {
    flex: 1;
  }

  .plot-text-mobile {
    font-size: 0.85rem;
    color: var(--color-text-muted);
    line-height: 1.5;
    margin: 0 0 var(--spacing-2) 0;
  }

  .meta-row-mobile {
    font-size: 0.8rem;
    display: flex;
    gap: var(--spacing-2);
  }

  .slide-up-enter-active,
  .slide-up-leave-active {
    transition: transform var(--transition-normal);
  }

  .slide-up-enter-from,
  .slide-up-leave-to {
    transform: translateY(100%);
  }
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity var(--transition-fast);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* Toast Messages */
.toast-message {
  position: fixed;
  bottom: 24px;
  right: 24px;
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: 12px 20px;
  border-radius: var(--radius-md);
  color: #fff;
  font-weight: 600;
  font-size: 0.9rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  z-index: 2000;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.toast-message.success {
  background-color: rgba(34, 197, 94, 0.9);
  border: 1px solid rgba(34, 197, 94, 0.2);
}

.toast-message.error {
  background-color: rgba(239, 68, 68, 0.9);
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.toast-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

.toast-fade-enter-active,
.toast-fade-leave-active {
  transition: all 0.25s ease-out;
}

.toast-fade-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.toast-fade-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}

@media (max-width: 768px) {
  .toast-message {
    left: 50%;
    right: auto;
    transform: translateX(-50%);
    bottom: 80px;
  }

  .toast-fade-enter-from {
    transform: translate(-50%, 20px);
  }
  .toast-fade-leave-to {
    transform: translate(-50%, -20px);
  }
}
</style>
