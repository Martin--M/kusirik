<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useLiveCategories, useLiveStreams } from '@/composables/useLiveStreams'
import { usePlayer } from '@/composables/usePlayer'
import LiveChannelList from '@/components/live/LiveChannelList.vue'
import CachedImage from '@/components/ui/CachedImage.vue'
import type { LiveStream, LiveCategory } from '@/types/stream'
import { invoke } from '@tauri-apps/api/core'
import { useProfileStore } from '@/stores/profile.store'
import { useSettingsStore } from '@/stores/settings.store'
import { getSetting, copyToSystemClipboard } from '@/lib/tauri-commands'
import { buildLiveUrl } from '@/lib/url-builder'

const selectedCategoryId = ref<string>('all')
const selectedStream = ref<LiveStream | null>(null)
const searchQuery = ref('')
const sortField = ref<'name' | 'tv_archive'>('name')
const sortOrder = ref<'asc' | 'desc'>('asc')

const isMobileDetailOpen = ref(false)

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

  // Apply sort
  return [...result].sort((a, b) => {
    if (sortField.value === 'name') {
      const nameA = a.name || ''
      const nameB = b.name || ''
      return sortOrder.value === 'asc'
        ? nameA.localeCompare(nameB)
        : nameB.localeCompare(nameA)
    } else if (sortField.value === 'tv_archive') {
      const archA = a.tv_archive || 0
      const archB = b.tv_archive || 0
      if (archA !== archB) {
        return sortOrder.value === 'asc' ? archA - archB : archB - archA
      }
      // fallback to name alphabetical
      return (a.name || '').localeCompare(b.name || '')
    }
    return 0
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

function toggleSort() {
  sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
}

async function copyUrl(stream: LiveStream) {
  try {
    // Construct dummy URL to let handoff copy it in P6
    const serverUrl = 'http://example.com' // will be overwritten in copy command
    const dummyUrl = `${serverUrl}/live/user/pass/${stream.stream_id}.ts`
    await invoke('copy_to_clipboard', { url: dummyUrl })
    alert('URL copied to clipboard!')
  } catch (e) {
    console.error(e)
  }
}
</script>

<template>
  <div class="live-view-container">
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
    <section class="channels-section">
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
              placeholder="Search by name or channel ID..."
              class="search-input"
            />
            <button v-if="searchQuery" class="clear-search" @click="searchQuery = ''">×</button>
          </div>

          <div class="sort-controls">
            <select v-model="sortField" class="sort-select" aria-label="Sort Field">
              <option value="name">Alphabetical</option>
              <option value="tv_archive">Catch-up</option>
            </select>
            <button class="sort-direction-btn" @click="toggleSort" :title="`Sort Direction: ${sortOrder}`">
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

    <!-- Details Sidebar (Desktop Only) -->
    <aside class="details-sidebar desktop-only">
      <transition name="fade" mode="out-in">
        <div v-if="selectedStream" :key="selectedStream.stream_id" class="details-panel">
          <button class="close-details-btn" @click="selectedStream = null" title="Close Details">×</button>
          <div class="details-header">
            <div class="details-logo">
              <CachedImage
                :src="selectedStream.stream_icon"
                :alt="selectedStream.name || 'Channel Logo'"
                :fallback-text="selectedStream.name || ''"
              />
            </div>
            <h3 class="channel-name-title">{{ selectedStream.name || 'Unnamed Channel' }}</h3>
            <span v-if="selectedStream.tv_archive === 1" class="archive-text">
              ⏱ {{ selectedStream.tv_archive_duration }} Days Catch-up available
            </span>
          </div>

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

          <div class="action-buttons">
            <button class="btn btn-primary" @click="handlePlay(selectedStream)">
              <svg viewBox="0 0 24 24" fill="currentColor" class="btn-icon">
                <polygon points="5 3 19 12 5 21 5 3" />
              </svg>
              Play Channel
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
              <rect x="2" y="7" width="20" height="15" rx="2" ry="2" />
              <polyline points="17 2 12 7 7 2" />
            </svg>
          </div>
          <p>Select a channel to view EPG details and start playback.</p>
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
            <div class="details-logo mobile">
              <CachedImage
                :src="selectedStream.stream_icon"
                :alt="selectedStream.name || 'Channel Logo'"
                :fallback-text="selectedStream.name || ''"
              />
            </div>
            <div class="header-text">
              <h3 class="channel-name-title">{{ selectedStream.name || 'Unnamed Channel' }}</h3>
              <span v-if="selectedStream.tv_archive === 1" class="archive-text">
                ⏱ {{ selectedStream.tv_archive_duration }} Days Catch-up
              </span>
            </div>
          </div>

          <div class="epg-box mobile">
            <div class="epg-placeholder-card active">
              <div class="epg-time">Now Playing</div>
              <div class="epg-title">EPG Data Sync Pending</div>
              <div class="epg-desc">
                Program guides will populate here once the EPG sync has run.
              </div>
            </div>
          </div>

          <div class="action-buttons mobile">
            <button class="btn btn-primary" @click="handlePlay(selectedStream); isMobileDetailOpen = false">
              Play Channel
            </button>
            <button class="btn btn-secondary" @click="copyUrl(selectedStream)">
              Copy URL
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.live-view-container {
  display: flex;
  height: 100%;
  width: 100%;
  background-color: var(--color-bg);
  min-height: 0; /* Ensures proper scroll scaling */
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

.sort-select {
  padding: var(--spacing-2) 28px var(--spacing-2) var(--spacing-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background-color: rgba(15, 23, 42, 0.4);
  color: var(--color-text);
  font-family: inherit;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  outline: none;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%2394a3b8' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  background-size: 14px;
}

[data-theme='light'] .sort-select {
  background-color: rgba(255, 255, 255, 0.6);
}

.sort-direction-btn {
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

.sort-direction-btn:hover {
  color: var(--color-text);
  background-color: rgba(255, 255, 255, 0.05);
}

.sort-icon {
  width: 16px;
  height: 16px;
  transition: transform var(--transition-normal);
}

.sort-icon.reversed {
  transform: rotate(180deg);
}

/* Mobile Categories Chips */
.mobile-categories {
  display: flex;
  gap: var(--spacing-2);
  overflow-x: auto;
  padding-bottom: 2px;
  scrollbar-width: none; /* Hide scrollbar for clean chip list */
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

.list-wrapper {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
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

.details-logo {
  width: 80px;
  height: 80px;
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-md), 0 0 25px rgba(96, 165, 250, 0.1);
}

.channel-name-title {
  font-size: 1.15rem;
  color: var(--color-text);
  font-weight: 700;
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

.action-buttons {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-2);
  padding: var(--spacing-3) var(--spacing-4);
  font-family: inherit;
  font-size: 0.95rem;
  font-weight: 600;
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  transition: all var(--transition-fast);
  width: 100%;
}

.btn-primary {
  background-color: var(--color-primary);
  color: #fff;
  box-shadow: 0 4px 12px rgba(96, 165, 250, 0.2);
}

.btn-primary:hover {
  background-color: var(--color-primary-hover);
  transform: translateY(-1px);
}

.btn-secondary {
  background-color: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--color-border);
  color: var(--color-text);
}

.btn-secondary:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.btn-icon {
  width: 18px;
  height: 18px;
}

.no-selection {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--spacing-8);
  color: var(--color-text-muted);
  text-align: center;
  gap: var(--spacing-4);
}

.tv-art {
  width: 64px;
  height: 64px;
  opacity: 0.4;
}

/* Skeletons */
.loading-sidebar {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.skeleton-pill {
  height: 38px;
  background-color: rgba(255, 255, 255, 0.03);
  border-radius: var(--radius-md);
  animation: pulse 1.5s infinite;
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

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity var(--transition-fast);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
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

  .details-logo.mobile {
    width: 64px;
    height: 64px;
    border-radius: var(--radius-md);
  }

  .header-text {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .action-buttons.mobile {
    flex-direction: row;
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
</style>
