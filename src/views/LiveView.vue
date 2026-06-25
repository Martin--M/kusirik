<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useLiveCategories, useLiveStreams } from '@/composables/useLiveStreams'
import { usePlayer } from '@/composables/usePlayer'
import LiveChannelList from '@/components/live/LiveChannelList.vue'
import CategorySidebar from '@/components/ui/CategorySidebar.vue'
import FilterHeader from '@/components/ui/FilterHeader.vue'
import StreamDetailPanel from '@/components/ui/StreamDetailPanel.vue'
import type { LiveStream, LiveCategory } from '@/types/stream'
import { getSetting, copyToSystemClipboard } from '@/lib/tauri-commands'
import { useProfileStore } from '@/stores/profile.store'
import { useSettingsStore } from '@/stores/settings.store'
import { useToastStore } from '@/stores/toast.store'
import { buildLiveUrl } from '@/lib/url-builder'
import IconLive from '@/components/icons/IconLive.vue'

const selectedCategoryId = ref<string>('all')
const selectedStream = ref<LiveStream | null>(null)
const searchQuery = ref('')
const sortOrder = ref<'asc' | 'desc'>('asc')

const isMobileDetailOpen = ref(false)

const profileStore = useProfileStore()
const settingsStore = useSettingsStore()
const toastStore = useToastStore()

const { data: categoriesData, isLoading: isLoadingCategories } = useLiveCategories()

// Computed categories list including "All" and "Uncategorized"
const categories = computed<LiveCategory[]>(() => {
  const list: LiveCategory[] = [
    { profile_id: 1, category_id: 'all', category_name: 'All Channels' },
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

const { data: rawStreams, isLoading: isLoadingStreams } = useLiveStreams(streamsQueryId)

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
      toastStore.showToast('No active profile loaded.', 'error')
      return
    }

    const password = await getSetting('password')
    if (!password) {
      toastStore.showToast('Could not retrieve credentials.', 'error')
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
    toastStore.showToast('URL copied to clipboard!', 'success')
  } catch (e) {
    console.error(e)
    toastStore.showToast('Failed to copy URL.', 'error')
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
          search-placeholder="Search by name or channel ID..."
        />
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
    <StreamDetailPanel
      :stream="selectedStream"
      :is-mobile-open="isMobileDetailOpen"
      play-button-text="Play Channel"
      copy-button-text="Copy Stream URL"
      @close="closeDetails"
      @play="handlePlay(selectedStream!)"
      @copy="copyUrl(selectedStream!)"
    >
      <template #header-meta>
        <span v-if="selectedStream?.tv_archive === 1" class="archive-text">
          ⏱ {{ selectedStream.tv_archive_duration }} Days Catch-up available
        </span>
      </template>

      <template #header-meta-mobile>
        <span v-if="selectedStream?.tv_archive === 1" class="archive-text">
          ⏱ {{ selectedStream.tv_archive_duration }} Days Catch-up
        </span>
      </template>

      <!-- Program guide elements inside detail layout -->
      <div class="epg-box">
        <h4 class="section-subtitle">Program Guide</h4>
        
        <div class="epg-placeholder-card active">
          <div class="epg-time">Now Playing</div>
          <div class="epg-title">EPG Data Sync Pending</div>
          <div class="epg-desc">
            A full program guide will populate here once the EPG sync (xmltv) has run. Keep IPTV Helper open or start a sync in Settings.
          </div>
          <div class="progress-bar-placeholder"></div>
        </div>

        <div class="epg-placeholder-card upcoming">
          <div class="epg-time">Next Program</div>
          <div class="epg-title">Upcoming Guide Info</div>
          <div class="epg-desc">Details of the next scheduled broadcasts will be fetched automatically.</div>
        </div>
      </div>

      <template #no-selection>
        <div class="no-selection">
          <div class="tv-art">
            <IconLive />
          </div>
          <p>Select a channel to view EPG details and start playback.</p>
        </div>
      </template>
    </StreamDetailPanel>
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

.list-wrapper {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.archive-text {
  font-size: 0.8rem;
  color: #4ade80;
  background: rgba(34, 197, 94, 0.1);
  padding: 2px 10px;
  border-radius: 9999px;
  font-weight: 600;
}

.epg-box {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.section-subtitle {
  font-size: 0.85rem;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 700;
  margin-bottom: 2px;
}

.epg-placeholder-card {
  background-color: rgba(15, 23, 42, 0.3);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-4);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
}

.epg-placeholder-card.active {
  border-color: rgba(96, 165, 250, 0.25);
  background-color: rgba(96, 165, 250, 0.02);
}

.epg-time {
  font-size: 0.75rem;
  color: var(--color-primary);
  font-weight: 700;
}

.epg-placeholder-card.upcoming .epg-time {
  color: var(--color-text-muted);
}

.epg-title {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--color-text);
}

.epg-desc {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  line-height: 1.4;
}

.progress-bar-placeholder {
  height: 4px;
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: 9999px;
  margin-top: 4px;
  overflow: hidden;
  position: relative;
}

.progress-bar-placeholder::after {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  width: 40%;
  background-color: var(--color-primary);
  border-radius: 9999px;
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
</style>
