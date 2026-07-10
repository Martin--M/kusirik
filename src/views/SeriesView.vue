<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useSeriesCategories, useSeries } from '@/composables/useSeries'
import SeriesList from '@/components/series/SeriesList.vue'
import CategorySidebar from '@/components/ui/CategorySidebar.vue'
import FilterHeader from '@/components/ui/FilterHeader.vue'
import SeriesDetailPanel from '@/components/series/SeriesDetailPanel.vue'
import type { Series, SeriesCategory } from '@/types/series'
import { useToastStore } from '@/stores/toast.store'
import { useI18n } from '@/composables/useI18n'
import { useToggleFavorite } from '@/composables/useFavorites'

const selectedCategoryId = ref<string>('all')
const selectedSeries = ref<Series | null>(null)
const { toggle: toggleFav } = useToggleFavorite()

async function handleToggleFavorite(series: Series) {
  const originalState = selectedSeries.value?.is_favorite
  const nextState = originalState === 1 ? 0 : 1

  // Optimistic update
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
    // Revert on error
    if (selectedSeries.value && selectedSeries.value.series_id === series.series_id) {
      selectedSeries.value = {
        ...selectedSeries.value,
        is_favorite: originalState
      }
    }
    toastStore.showToast(t('media.favoriteToggleFailed') || 'Failed to update favorite status', 'error')
  }
}
const searchQuery = ref('')
const sortField = ref<'name' | 'rating'>('name')
const sortOrder = ref<'asc' | 'desc'>('asc')
const isGridView = ref(true)

const isMobileDetailOpen = ref(false)
const expandedSeason = ref<string | number | null>(null)

const { t } = useI18n()

const sortLabels = computed(() => ({
  name: t('sortField.name'),
  rating: t('sortField.rating')
}))

const toastStore = useToastStore()

import { useProfileStore } from '@/stores/profile.store'

const profileStore = useProfileStore()
const { data: categoriesData, isLoading: isLoadingCategories } = useSeriesCategories(
  computed(() => profileStore.filterProfileId)
)

// Computed categories list including "All" and "Uncategorized"
const categories = computed<SeriesCategory[]>(() => {
  const currentProfileId = profileStore.profile?.id
  if (currentProfileId === undefined) {
    return []
  }
  const list: SeriesCategory[] = [
    { profile_id: currentProfileId, category_id: 'all', category_name: t('media.allSeries') },
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


// Feed reactive categoryId to the series query
const seriesQueryId = computed(() => {
  if (selectedCategoryId.value === 'all') return undefined
  return selectedCategoryId.value
})

const { data: rawSeries, isLoading: isLoadingSeries } = useSeries(
  seriesQueryId,
  computed(() => profileStore.filterProfileId)
)

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

function selectSeries(series: Series) {
  selectedSeries.value = series
  isMobileDetailOpen.value = true
}

function closeDetails() {
  selectedSeries.value = null
  isMobileDetailOpen.value = false
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
          show-layout-toggle
          :search-placeholder="$t('media.searchSeries')"
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
    <!-- Details Sidebar (Desktop Only) & Bottom Sheet (Mobile Only) -->
    <SeriesDetailPanel
      :series="selectedSeries"
      :is-mobile-open="isMobileDetailOpen"
      @close="closeDetails"
      @toggle-favorite="handleToggleFavorite"
    />
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

</style>
