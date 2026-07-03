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
import { buildLiveUrl, buildCatchupUrl } from '@/lib/url-builder'
import { useI18n } from '@/composables/useI18n'
import IconLive from '@/components/icons/IconLive.vue'
import IconCheck from '@/components/icons/IconCheck.vue'
import IconPlay from '@/components/icons/IconPlay.vue'
import IconCopy from '@/components/icons/IconCopy.vue'

const selectedCategoryId = ref<string>('all')
const selectedStream = ref<LiveStream | null>(null)
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

const { playLive, playCatchup } = usePlayer()

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

// ─── EPG guide setup ─────────────────────────────────────────────────────────
import { useEpg } from '@/composables/useEpg'

const selectedEpgChannelId = computed(() => selectedStream.value?.epg_channel_id || null)

const hoursBack = computed(() => {
  if (selectedStream.value && selectedStream.value.tv_archive === 1) {
    return (selectedStream.value.tv_archive_duration || 1) * 24
  }
  return 1
})

const { data: epgData, isLoading: isLoadingEpg } = useEpg(selectedEpgChannelId, hoursBack)

const now = ref(new Date())
// Update "now" timer to refresh progress bar
const timer = setInterval(() => {
  now.value = new Date()
}, 30000)

// Clean up timer on unmount
import { onUnmounted } from 'vue'
onUnmounted(() => {
  clearInterval(timer)
})

const currentProgram = computed(() => {
  if (!epgData.value) return null
  return epgData.value.find((entry) => {
    const start = new Date(entry.start)
    const stop = new Date(entry.stop)
    return start <= now.value && stop >= now.value
  }) || null
})

const pastPrograms = computed(() => {
  if (!epgData.value || !selectedStream.value || selectedStream.value.tv_archive !== 1) return []
  const durationMs = (selectedStream.value.tv_archive_duration || 0) * 24 * 3_600_000
  const cutoffTime = new Date(now.value.getTime() - durationMs)
  
  const finished = epgData.value.filter((entry) => {
    const start = new Date(entry.start)
    const stop = new Date(entry.stop)
    return stop < now.value && start >= cutoffTime
  })
  
  return [...finished].sort((a, b) => new Date(b.start).getTime() - new Date(a.start).getTime())
})

function formatUtcForCatchup(dateStr: string, tzOffset?: string | null): string {
  try {
    let date = new Date(dateStr)
    if (tzOffset) {
      const sign = tzOffset.startsWith('-') ? -1 : 1
      const cleaned = tzOffset.replace(/[+-]/g, '')
      const parts = cleaned.split(':')
      if (parts.length === 2) {
        const hours = parseInt(parts[0], 10)
        const minutes = parseInt(parts[1], 10)
        const totalOffsetMinutes = sign * (hours * 60 + minutes)
        date = new Date(date.getTime() + totalOffsetMinutes * 60 * 1000)
      }
    }
    const y = date.getUTCFullYear()
    const m = String(date.getUTCMonth() + 1).padStart(2, '0')
    const d = String(date.getUTCDate()).padStart(2, '0')
    const h = String(date.getUTCHours()).padStart(2, '0')
    const min = String(date.getUTCMinutes()).padStart(2, '0')
    return `${y}-${m}-${d}:${h}-${min}`
  } catch (e) {
    return ''
  }
}

function getDurationMinutes(startStr: string, stopStr: string): number {
  try {
    const start = new Date(startStr).getTime()
    const stop = new Date(stopStr).getTime()
    return Math.round((stop - start) / 60000)
  } catch (e) {
    return 0
  }
}

const isCatchupValid = ref(false)
let lastCheckStreamId: number | null = null

watch(selectedStream, () => {
  isCatchupValid.value = false
  lastCheckStreamId = null
})

watch(pastPrograms, async (newPastPrograms) => {
  if (!newPastPrograms || newPastPrograms.length === 0 || !selectedStream.value) {
    isCatchupValid.value = false
    return
  }

  // Prevent redundant checks if we already successfully validated this channel
  if (isCatchupValid.value && lastCheckStreamId === selectedStream.value.stream_id) {
    return
  }

  const streamId = selectedStream.value.stream_id
  lastCheckStreamId = streamId
  isCatchupValid.value = false

  try {
    const profile = profileStore.profile
    if (!profile) return

    const password = await getSetting('password')
    if (!password) return

    // Precheck using a real, already-aired program from EPG (guaranteed valid time window)
    const targetProgram = newPastPrograms[0]
    const startDateTime = formatUtcForCatchup(targetProgram.start, targetProgram.tz_offset)
    const duration = getDurationMinutes(targetProgram.start, targetProgram.stop)

    const url = buildCatchupUrl(
      {
        serverUrl: profile.server_url,
        username: profile.username,
        password: '***',
      },
      streamId,
      startDateTime,
      duration
    )

    const { validateStreamUrl } = await import('@/lib/tauri-commands')
    const valid = await validateStreamUrl(url)

    if (lastCheckStreamId === streamId) {
      isCatchupValid.value = valid
    }
  } catch (err: any) {
    console.error('Catchup pre-check error:', err)
    if (lastCheckStreamId === streamId) {
      isCatchupValid.value = false
      const errorMsg = typeof err === 'string' ? err : (err?.message || JSON.stringify(err))
      toastStore.showToast(`${t('media.catchupUnreachable')} (${errorMsg})`, 'error')
    }
  }
}, { immediate: true })

async function handlePlayCatchup(item: any) {
  if (!selectedStream.value) return
  const startDateTime = formatUtcForCatchup(item.start, item.tz_offset)
  const duration = getDurationMinutes(item.start, item.stop)
  await playCatchup(selectedStream.value.stream_id, startDateTime, duration)
}

async function copyCatchupUrl(item: any) {
  try {
    const profile = profileStore.profile
    if (!profile || !selectedStream.value) {
      toastStore.showToast(t('settings.profile.disconnectFailed', { error: 'No profile' }), 'error')
      return
    }

    const password = await getSetting('password')
    if (!password) {
      toastStore.showToast(t('setup.saveFailed', { error: 'Credentials' }), 'error')
      return
    }

    const startDateTime = formatUtcForCatchup(item.start, item.tz_offset)
    const duration = getDurationMinutes(item.start, item.stop)
    const url = buildCatchupUrl(
      {
        serverUrl: profile.server_url,
        username: profile.username,
        password,
      },
      selectedStream.value.stream_id,
      startDateTime,
      duration
    )

    await copyToSystemClipboard(url)
    toastStore.showToast(t('media.urlCopied'), 'success')
  } catch (e) {
    console.error(e)
    toastStore.showToast(t('media.copyFailed'), 'error')
  }
}

const currentProgramProgress = computed(() => {
  if (!currentProgram.value) return 0
  const start = new Date(currentProgram.value.start).getTime()
  const stop = new Date(currentProgram.value.stop).getTime()
  const current = now.value.getTime()
  if (stop === start) return 0
  const progress = ((current - start) / (stop - start)) * 100
  return Math.max(0, Math.min(100, progress))
})

const upcomingPrograms = computed(() => {
  if (!epgData.value) return []
  return epgData.value.filter((entry) => {
    const start = new Date(entry.start)
    return start > now.value
  })
})

function formatEpgTime(dateStr: string): string {
  try {
    const date = new Date(dateStr)
    const h = String(date.getHours()).padStart(2, '0')
    const m = String(date.getMinutes()).padStart(2, '0')
    return `${h}:${m}`
  } catch (e) {
    return ''
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
    <StreamDetailPanel
      :stream="selectedStream"
      :is-mobile-open="isMobileDetailOpen"
      square-image
      :default-width="500"
      :play-button-text="$t('media.play')"
      :copy-button-text="$t('media.copyUrl')"
      @close="closeDetails"
      @play="handlePlay(selectedStream!)"
      @copy="copyUrl(selectedStream!)"
    >
      <template #header-meta>
        <span v-if="selectedStream?.tv_archive === 1" class="archive-text">
          ⏱ {{ $t('media.catchupDays', { days: selectedStream.tv_archive_duration }) }}
        </span>
      </template>

      <template #header-meta-mobile>
        <span v-if="selectedStream?.tv_archive === 1" class="archive-text">
          ⏱ {{ $t('media.catchupDays', { days: selectedStream.tv_archive_duration }) }}
        </span>
      </template>

      <!-- Program guide elements inside detail layout -->
      <div class="epg-box">
        <div v-if="isLoadingEpg" class="epg-loading">
          <span class="spinner small"></span>
          <span>{{ $t('settings.stats.loading') }}</span>
        </div>

        <div v-else-if="!currentProgram && upcomingPrograms.length === 0 && pastPrograms.length === 0" class="epg-no-data">
          <p>{{ $t('media.noEpg') }}</p>
        </div>

        <div v-else>
          <!-- Past Schedule (Catch-up) -->
          <div v-if="selectedStream?.tv_archive === 1 && pastPrograms.length > 0" class="epg-past-section">
            <h5 class="epg-past-header">{{ $t('media.pastSchedule') }}</h5>
            <div class="epg-past-list">
              <div v-for="item in pastPrograms" :key="item.id || item.start" class="epg-past-item">
                <div class="epg-past-info">
                  <span class="epg-past-time">{{ formatEpgTime(item.start) }} - {{ formatEpgTime(item.stop) }}</span>
                  <span class="epg-past-title" :title="item.title || undefined">{{ item.title }}</span>
                </div>
                <div class="epg-past-actions">
                  <button class="action-btn play-btn" :disabled="!isCatchupValid" @click.stop="handlePlayCatchup(item)" :title="$t('media.playCatchup')">
                    <IconPlay class="action-icon" />
                  </button>
                  <button class="action-btn" @click.stop="copyCatchupUrl(item)" :title="$t('media.copyUrl')">
                    <IconCopy class="action-icon" />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Now Playing Program Card -->
          <div v-if="currentProgram" class="epg-placeholder-card active">
            <div class="epg-time">
              {{ $t('media.nowPlaying') }} ({{ formatEpgTime(currentProgram.start) }} - {{ formatEpgTime(currentProgram.stop) }})
            </div>
            <div class="epg-title">{{ currentProgram.title }}</div>
            <div v-if="currentProgram.description" class="epg-desc">
              {{ currentProgram.description }}
            </div>
            <div class="progress-bar-placeholder" :title="`${Math.round(currentProgramProgress)}% elapsed`">
              <div class="progress-bar-fill" :style="{ width: `${currentProgramProgress}%` }"></div>
            </div>
          </div>

          <!-- Upcoming Programs List (Scrollable) -->
          <div v-if="upcomingPrograms.length > 0" class="epg-upcoming-section">
            <h5 class="epg-upcoming-header">{{ $t('media.upcomingSchedule') }}</h5>
            <div class="epg-upcoming-list">
              <div v-for="item in upcomingPrograms" :key="item.id || item.start" class="epg-upcoming-item">
                <div class="epg-upcoming-time">
                  {{ formatEpgTime(item.start) }} - {{ formatEpgTime(item.stop) }}
                </div>
                <div class="epg-upcoming-title">{{ item.title }}</div>
                <div v-if="item.description" class="epg-upcoming-desc">{{ item.description }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <template #no-selection>
        <div class="no-selection">
          <div class="tv-art">
            <IconLive />
          </div>
          <p>{{ $t('media.selectToView') }}</p>
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
  background-color: rgba(255, 255, 255, 0.08);
  border-radius: 9999px;
  margin-top: 6px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background-color: var(--color-primary);
  border-radius: 9999px;
  transition: width 0.3s ease;
}

.epg-loading {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  color: var(--color-text-muted);
  font-size: 0.85rem;
  padding: var(--spacing-4) 0;
  justify-content: center;
}

.epg-no-data {
  color: var(--color-text-muted);
  font-size: 0.85rem;
  padding: var(--spacing-4) 0;
  text-align: center;
  background: rgba(15, 23, 42, 0.15);
  border: 1px dashed var(--color-border);
  border-radius: var(--radius-md);
}

.epg-upcoming-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
  margin-top: var(--spacing-4);
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-4);
}

.epg-upcoming-header {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 700;
  margin-bottom: var(--spacing-1);
}

.epg-upcoming-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
  max-height: 320px;
  overflow-y: auto;
  padding-right: var(--spacing-2);
}

.epg-upcoming-item {
  background: rgba(15, 23, 42, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.03);
  border-left: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-3);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-1);
  transition: border-color var(--transition-fast) ease, background-color var(--transition-fast) ease;
}

.epg-upcoming-item:hover {
  background: rgba(15, 23, 42, 0.35);
  border-left-color: var(--color-primary);
}

.epg-upcoming-time {
  font-size: 0.75rem;
  color: var(--color-primary);
  font-weight: 700;
}

.epg-upcoming-title {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text);
}

.epg-upcoming-desc {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  line-height: 1.4;
  margin-top: 2px;
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

/* Past Schedule Catch-up Styles */
.epg-past-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
  margin-bottom: var(--spacing-2);
}

.epg-past-header {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 700;
  margin-bottom: var(--spacing-1);
}

.epg-past-list {
  display: flex;
  flex-direction: column-reverse;
  gap: var(--spacing-1-5);
  max-height: 200px;
  overflow-y: auto;
  padding-right: var(--spacing-1);
}

.epg-past-item {
  background: rgba(15, 23, 42, 0.15);
  border: 1px solid rgba(255, 255, 255, 0.02);
  border-radius: var(--radius-sm);
  padding: var(--spacing-2) var(--spacing-3);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-3);
  font-size: 0.8rem;
}

.epg-past-item:hover {
  background: rgba(15, 23, 42, 0.25);
  border-color: rgba(255, 255, 255, 0.05);
}

.epg-past-info {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  min-width: 0;
  flex: 1;
}

.epg-past-time {
  font-weight: 700;
  color: var(--color-primary);
  flex-shrink: 0;
}

.epg-past-title {
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.epg-past-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  flex-shrink: 0;
}

.action-btn {
  background: transparent;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color var(--transition-fast) ease, background-color var(--transition-fast) ease;
}

.action-btn:hover {
  color: var(--color-text);
  background-color: rgba(255, 255, 255, 0.06);
}

.action-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
  pointer-events: none;
}

.action-btn.play-btn {
  color: var(--color-primary);
}

.action-btn.play-btn:hover {
  color: var(--color-primary-hover);
  background-color: rgba(59, 130, 246, 0.15);
}

.action-icon {
  width: 14px;
  height: 14px;
}
</style>
