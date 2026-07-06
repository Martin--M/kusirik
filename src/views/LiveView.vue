<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useLiveCategories, useLiveStreams } from '@/composables/useLiveStreams'
import { usePlayer } from '@/composables/usePlayer'
import LiveChannelList from '@/components/live/LiveChannelList.vue'
import CategorySidebar from '@/components/ui/CategorySidebar.vue'
import FilterHeader from '@/components/ui/FilterHeader.vue'
import LiveDetailPanel from '@/components/live/LiveDetailPanel.vue'
import type { LiveStream, LiveCategory } from '@/types/stream'
import { getSetting, copyToSystemClipboard } from '@/lib/tauri-commands'
import { useProfileStore } from '@/stores/profile.store'
import { useSettingsStore } from '@/stores/settings.store'
import { useToastStore } from '@/stores/toast.store'
import { buildLiveUrl } from '@/lib/url-builder'
import { useI18n } from '@/composables/useI18n'
import IconCheck from '@/components/icons/IconCheck.vue'
import { useToggleFavorite } from '@/composables/useFavorites'

const selectedCategoryId = ref<string>('all')
const selectedStream = ref<LiveStream | null>(null)
const { toggle: toggleFav } = useToggleFavorite()

async function handleToggleFavorite(stream: LiveStream) {
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
    const isFav = await toggleFav('live', stream.stream_id)
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
const sortOrder = ref<'asc' | 'desc'>('asc')
const showCatchupOnly = ref(false)

const isMobileDetailOpen = ref(false)

const profileStore = useProfileStore()
const settingsStore = useSettingsStore()
const toastStore = useToastStore()
const { t } = useI18n()

const { data: categoriesData, isLoading: isLoadingCategories } = useLiveCategories()

// Computed categories list including "All" and "Uncategorized"
const categories = computed<LiveCategory[]>(() => {
  const list: LiveCategory[] = [
    { profile_id: 1, category_id: 'all', category_name: t('media.allChannels') },
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

const { data: rawStreams, isLoading: isLoadingStreams } = useLiveStreams(streamsQueryId)

// Reset selection when changing categories or toggle filters
watch(selectedCategoryId, () => {
  selectedStream.value = null
})

watch(showCatchupOnly, () => {
  selectedStream.value = null
})

// Filter and sort streams on client side
const filteredStreams = computed(() => {
  let result = rawStreams.value || []

  // Apply catch-up filter
  if (showCatchupOnly.value) {
    result = result.filter((s) => s.tv_archive === 1)
  }

  // Apply search query
  const query = searchQuery.value.toLowerCase().trim()
  if (query) {
    result = result.filter(
      (s) =>
        (s.name && s.name.toLowerCase().includes(query)) ||
        (s.epg_channel_id && s.epg_channel_id.toLowerCase().includes(query))
    )
  }

  // Apply sort (Alphabetical only)
  return [...result].sort((a, b) => {
    const nameA = a.name || ''
    const nameB = b.name || ''
    return sortOrder.value === 'asc'
      ? nameA.localeCompare(nameB)
      : nameB.localeCompare(nameA)
  })
})

const { playLive } = usePlayer()

function selectChannel(stream: LiveStream) {
  selectedStream.value = stream
  isMobileDetailOpen.value = true
}

function handlePlay(stream: LiveStream) {
  playLive(stream.stream_id)
}

function closeDetails() {
  selectedStream.value = null
  isMobileDetailOpen.value = false
}

async function copyUrl(stream: LiveStream) {
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

    const format = settingsStore.liveFormat || 'ts'
    const url = buildLiveUrl(
      {
        serverUrl: profile.server_url,
        username: profile.username,
        password,
      },
      stream.stream_id,
      format
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
  <div class="live-view-container">
    <!-- Categories Sidebar (Desktop Only) -->
    <CategorySidebar
      layout="sidebar"
      :categories="categories"
      :selected-id="selectedCategoryId"
      :is-loading="isLoadingCategories"
      @select="selectedCategoryId = $event"
    />

    <!-- Main List/Grid & Filters Area -->
    <section class="channels-section">
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
          v-model:sort-order="sortOrder"
          :search-placeholder="$t('media.searchChannels')"
        >
          <label class="custom-checkbox">
            <input type="checkbox" v-model="showCatchupOnly" class="checkbox-input" />
            <span class="checkbox-box">
              <IconCheck class="checkbox-check" />
            </span>
            <span class="checkbox-label">{{ $t('media.catchupOnly') }}</span>
          </label>
        </FilterHeader>
      </div>

      <!-- Virtualized Scroll List -->
      <div class="list-wrapper">
        <div v-if="isLoadingStreams" class="loading-streams">
          <div v-for="i in 6" :key="i" class="skeleton-row">
            <div class="skeleton-avatar"></div>
            <div class="skeleton-info">
              <div class="skeleton-line short"></div>
              <div class="skeleton-line"></div>
            </div>
          </div>
        </div>
        <LiveChannelList
          v-else
          :streams="filteredStreams"
          :selected-stream-id="selectedStream?.stream_id || null"
          @select="selectChannel"
          @play="handlePlay"
        />
      </div>
    </section>

    <!-- Details Sidebar (Desktop Only) & Sheet (Mobile Only) -->
    <LiveDetailPanel
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
.live-view-container {
  display: flex;
  height: 100%;
  width: 100%;
  background-color: var(--color-bg);
  min-height: 0;
}

.channels-section {
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

/* Custom Checkbox Filter */
.custom-checkbox {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-2);
  cursor: pointer;
  user-select: none;
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text-muted);
  transition: color var(--transition-fast) ease;
  margin-right: var(--spacing-2);
}

.custom-checkbox:hover {
  color: var(--color-text);
}

.checkbox-input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.checkbox-box {
  width: 18px;
  height: 18px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border);
  background-color: rgba(15, 23, 42, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-fast) ease;
}

[data-theme='light'] .checkbox-box {
  background-color: rgba(255, 255, 255, 0.6);
}

.custom-checkbox:hover .checkbox-box {
  border-color: var(--color-primary);
  background-color: rgba(255, 255, 255, 0.05);
}

[data-theme='light'] .custom-checkbox:hover .checkbox-box {
  background-color: rgba(0, 0, 0, 0.02);
}

.checkbox-input:checked + .checkbox-box {
  background-color: var(--color-primary);
  border-color: var(--color-primary);
}

.checkbox-check {
  width: 12px;
  height: 12px;
  color: #ffffff;
  stroke-dasharray: 30;
  stroke-dashoffset: 30;
  opacity: 0;
  transition: stroke-dashoffset 0.15s ease-in-out, opacity 0.15s ease-in-out;
}

.checkbox-input:checked + .checkbox-box .checkbox-check {
  stroke-dashoffset: 0;
  opacity: 1;
}

.checkbox-label {
  white-space: nowrap;
}

.list-wrapper {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.loading-streams {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-4);
  gap: var(--spacing-4);
}

.skeleton-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-4);
  opacity: 0.6;
}

.skeleton-avatar {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-md);
  background-color: rgba(255, 255, 255, 0.04);
  animation: pulse 1.5s infinite;
}

.skeleton-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.skeleton-line {
  height: 12px;
  background-color: rgba(255, 255, 255, 0.04);
  border-radius: var(--radius-sm);
  animation: pulse 1.5s infinite;
}

.skeleton-line.short {
  width: 40%;
}

@keyframes pulse {
  0% { opacity: 0.4; }
  50% { opacity: 0.8; }
  100% { opacity: 0.4; }
}

.live-header-meta-row {
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
</style>
