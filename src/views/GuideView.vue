<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { useInfiniteQuery, keepPreviousData } from '@tanstack/vue-query'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { getEpgGuide, getSyncStatus } from '@/lib/tauri-commands'
import { usePlayer } from '@/composables/usePlayer'
import { useI18n } from '@/composables/useI18n'
import { useToastStore } from '@/stores/toast.store'
import CachedImage from '@/components/ui/CachedImage.vue'
import LiveDetailPanel from '@/components/live/LiveDetailPanel.vue'
import IconCheck from '@/components/icons/IconCheck.vue'
import CustomSelect from '@/components/ui/CustomSelect.vue'
import { buildLiveUrl, buildCatchupUrl } from '@/lib/url-builder'
import {
  formatUtcForCatchup,
  getDurationMinutes,
  isCurrentProgram as checkCurrentProgram,
  isPastProgram as checkPastProgram
} from '@/lib/date-utils'
import { useProfileStore } from '@/stores/profile.store'
import type { EpgEntry } from '@/types/epg'

import { useToggleFavorite } from '@/composables/useFavorites'
import { checkIsDesktop } from '@/lib/device'

const { t } = useI18n()
const toastStore = useToastStore()
const profileStore = useProfileStore()
const { playLive, playCatchup } = usePlayer()
const { toggle: toggleFav } = useToggleFavorite()

async function handleToggleFavorite(channel: any) {
  const originalState = selectedChannel.value?.is_favorite
  const nextState = originalState === 1 ? 0 : 1

  if (selectedChannel.value && selectedChannel.value.stream_id === channel.stream_id) {
    selectedChannel.value = {
      ...selectedChannel.value,
      is_favorite: nextState
    }
  }

  try {
    const isFav = await toggleFav('live', channel.stream_id)
    if (selectedChannel.value && selectedChannel.value.stream_id === channel.stream_id) {
      selectedChannel.value = {
        ...selectedChannel.value,
        is_favorite: isFav ? 1 : 0
      }
    }
  } catch (err) {
    console.error('Failed to toggle favorite:', err)
    if (selectedChannel.value && selectedChannel.value.stream_id === channel.stream_id) {
      selectedChannel.value = {
        ...selectedChannel.value,
        is_favorite: originalState
      }
    }
    toastStore.showToast(t('media.favoriteToggleFailed') || 'Failed to update favorite status', 'error')
  }
}

// Setup time window parameters
const pxPerMinute = 4
const PAGE_SIZE = 500

const now = ref(new Date())
const baseTime = ref(new Date())

const showCatchupOnly = ref(false)

const selectedProfileId = ref<string>('all')

const selectedCountry = ref<string>('all')

const profileOptions = computed(() => {
  const opts: Record<string, string> = {
    'all': t('media.allProfiles')
  }
  for (const p of profileStore.profiles) {
    opts[String(p.id)] = p.name
  }
  return opts
})


import { getCountryName } from '@/lib/countries'

const countryOptions = computed(() => {
  const opts: Record<string, string> = {
    'all': t('media.allCountries')
  }
  const countries = new Set<string>()
  const list = channels.value || []
  for (const c of list) {
    if (c.countries) {
      c.countries.split(',').forEach(cntry => {
        const cleaned = cntry.trim()
        if (cleaned) {
          countries.add(cleaned.toUpperCase())
        }
      })
    }
  }
  const mapped = Array.from(countries).map(c => ({
    code: c,
    name: getCountryName(c)
  }))
  mapped.sort((a, b) => a.name.localeCompare(b.name))
  mapped.forEach(item => {
    opts[item.code] = item.name
  })
  return opts
})

const streamsQueryProfileId = computed(() => {
  if (selectedProfileId.value === 'all') return undefined
  return Number(selectedProfileId.value)
})

watch(selectedProfileId, () => {
  closeDetails()
  selectedCountry.value = 'all'
})

watch(selectedCountry, () => {
  closeDetails()
})

const isTimeCentered = ref(false)
const startTimeRef = ref<Date>(new Date())
const endTimeRef = ref<Date>(new Date())

function resetTimeRange() {
  isTimeCentered.value = false
  if (showCatchupOnly.value) {
    startTimeRef.value = new Date(baseTime.value.getTime() - 24 * 3600 * 1000)
    endTimeRef.value = new Date(baseTime.value.getTime() + 1 * 3600 * 1000)
  } else {
    startTimeRef.value = new Date(baseTime.value.getTime() - 1 * 3600 * 1000)
    endTimeRef.value = new Date(baseTime.value.getTime() + 4 * 3600 * 1000)
  }
}

watch([baseTime, showCatchupOnly], resetTimeRange, { immediate: true })

const startTime = computed(() => startTimeRef.value)
const endTime = computed(() => endTimeRef.value)

const queryStartTime = computed(() => {
  const d = new Date(startTime.value.getTime() - 2 * 3600 * 1000)
  d.setHours(Math.floor(d.getHours() / 6) * 6, 0, 0, 0)
  return d.toISOString()
})

const queryEndTime = computed(() => {
  const d = new Date(endTime.value.getTime() + 2 * 3600 * 1000)
  d.setHours(Math.ceil(d.getHours() / 6) * 6, 0, 0, 0)
  return d.toISOString()
})

const totalTimelineMinutes = computed(() => {
  return (endTime.value.getTime() - startTime.value.getTime()) / 60000
})

const lastEpgFetchedAt = ref<string | null>(null)
const isMounted = ref(false)

// Periodic timer to keep current time line updated and check for EPG sync changes
let timer: any = null
onMounted(async () => {
  isMounted.value = true
  // Capture initial fetched_at
  try {
    const profileId = profileStore.profile?.id
    if (profileId !== undefined) {
      const statuses = await getSyncStatus(profileId)
      const epgStatus = statuses.find(s => s.data_type === 'epg')
      if (epgStatus) {
        lastEpgFetchedAt.value = epgStatus.fetched_at
      }
    }
  } catch (err) {
    console.error('Failed to get EPG sync status on mount', err)
  }

  timer = setInterval(async () => {
    now.value = new Date()
    
    // Check if new EPG dataset has been fetched
    try {
      const profileId = profileStore.profile?.id
      if (profileId !== undefined) {
        const statuses = await getSyncStatus(profileId)
        const epgStatus = statuses.find(s => s.data_type === 'epg')
        if (epgStatus && epgStatus.fetched_at !== lastEpgFetchedAt.value) {
          lastEpgFetchedAt.value = epgStatus.fetched_at
          // Re-center window and re-query
          baseTime.value = new Date()
        }
      }
    } catch (err) {
      console.error('Failed to check sync status in timer', err)
    }
  }, 60000)
  
  // Auto scroll to current time marker
  setTimeout(() => {
    scrollToNow()
  }, 300)
})

import { onUnmounted } from 'vue'
onUnmounted(() => {
  if (timer) clearInterval(timer)
})

// Query EPG Guide data via tauri command
const {
  data: infiniteData,
  fetchNextPage,
  hasNextPage,
  isFetchingNextPage,
  isLoading,
  error
} = useInfiniteQuery({
  queryKey: ['epg_guide', queryStartTime, queryEndTime, () => profileStore.profiles.map(p => p.id), streamsQueryProfileId],
  queryFn: async ({ pageParam = 0 }) => {
    const fromStr = queryStartTime.value
    const toStr = queryEndTime.value
    if (profileStore.profiles.length === 0) {
      throw new Error('Cannot query EPG Guide: No profiles are configured.')
    }
    return await getEpgGuide(streamsQueryProfileId.value, fromStr, toStr, pageParam, PAGE_SIZE)
  },
  initialPageParam: 0,
  getNextPageParam: (lastPage, allPages) => {
    if (!lastPage || lastPage.length < PAGE_SIZE) {
      return undefined
    }
    return allPages.length * PAGE_SIZE
  },
  refetchInterval: 300000, // 5 minutes refresh
  placeholderData: keepPreviousData,
  enabled: isMounted
})

const channels = computed(() => {
  if (!infiniteData.value) return []
  return infiniteData.value.pages.flat()
})


function deduplicateEpgEntries(entries: EpgEntry[]): EpgEntry[] {
  if (!entries || entries.length === 0) return []
  const sorted = entries.map(e => ({ ...e })).sort((a, b) => new Date(a.start).getTime() - new Date(b.start).getTime())
  const result: EpgEntry[] = []
  
  for (const entry of sorted) {
    if (result.length === 0) {
      result.push(entry)
      continue
    }
    
    const last = result[result.length - 1]
    const lastStart = new Date(last.start).getTime()
    const lastStop = new Date(last.stop).getTime()
    const currentStart = new Date(entry.start).getTime()
    
    // Check if duplicate (same start time or title and very close start times)
    if (lastStart === currentStart || (last.title === entry.title && Math.abs(lastStart - currentStart) < 300000)) {
      continue
    }
    
    // If current start is before last stop, they overlap
    if (currentStart < lastStop) {
      if (currentStart > lastStart) {
        last.stop = entry.start
      } else {
        continue
      }
    }
    
    result.push(entry)
  }
  
  return mergeShortSegments(result)
}

function mergeShortSegments(entries: EpgEntry[]): EpgEntry[] {
  if (entries.length <= 1) return entries
  
  let i = 0
  while (i < entries.length) {
    const entry = entries[i]
    const duration = (new Date(entry.stop).getTime() - new Date(entry.start).getTime()) / 60000
    
    if (duration <= 5) {
      let merged = false
      const prev = i > 0 ? entries[i - 1] : null
      const next = i < entries.length - 1 ? entries[i + 1] : null
      
      const prevDuration = prev ? (new Date(prev.stop).getTime() - new Date(prev.start).getTime()) / 60000 : Infinity
      const nextDuration = next ? (new Date(next.stop).getTime() - new Date(next.start).getTime()) / 60000 : Infinity
      
      if (prevDuration <= 5) {
        prev!.stop = entry.stop
        entries.splice(i, 1)
        merged = true
      } else if (nextDuration <= 5) {
        next!.start = entry.start
        entries.splice(i, 1)
        merged = true
      } else {
        if (prev) {
          prev.stop = entry.stop
          entries.splice(i, 1)
          merged = true
        } else if (next) {
          next.start = entry.start
          entries.splice(i, 1)
          merged = true
        }
      }
      
      if (merged) {
        continue
      }
    }
    i++
  }
  return entries
}

const cleanChannels = computed(() => {
  if (!channels.value) return []
  return channels.value.map(channel => ({
    ...channel,
    epg_entries: deduplicateEpgEntries(channel.epg_entries)
  }))
})

const searchQuery = ref('')
const filteredChannels = computed(() => {
  let list = cleanChannels.value
  
  if (showCatchupOnly.value) {
    list = list.filter(channel => channel.tv_archive === 1)
  }



  // Apply country filter
  if (selectedCountry.value !== 'all') {
    const country = selectedCountry.value.toLowerCase()
    list = list.filter(c => c.countries?.toLowerCase().split(',').map(cntry => cntry.trim()).includes(country))
  }

  // Filter EPG entries to only include those overlapping the current render window
  const startMs = startTime.value.getTime()
  const endMs = endTime.value.getTime()

  list = list.map(channel => {
    const visibleEntries = channel.epg_entries.filter(entry => {
      const entryStart = new Date(entry.start).getTime()
      const entryStop = new Date(entry.stop).getTime()
      return entryStart < endMs && entryStop > startMs
    })
    return {
      ...channel,
      epg_entries: visibleEntries
    }
  })

  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return list

  return list.filter(channel => {
    const channelMatch = (channel.name || '').toLowerCase().includes(query)
    const programMatch = channel.epg_entries.some(entry =>
      (entry.title || '').toLowerCase().includes(query)
    )
    return channelMatch || programMatch
  })
})

// Calculate list of 30-minute interval columns
const timeSlots = computed(() => {
  const slots = []
  const current = new Date(startTime.value)
  // Round to nearest half hour
  current.setMinutes(current.getMinutes() >= 30 ? 30 : 0, 0, 0)
  
  while (current < endTime.value) {
    slots.push(new Date(current))
    current.setMinutes(current.getMinutes() + 30)
  }
  return slots
})

// Timeline horizontal positioning helper
function getPositionLeft(dateStr: string): number {
  const time = new Date(dateStr).getTime()
  const start = startTime.value.getTime()
  const diffMinutes = (time - start) / 60000
  return diffMinutes * pxPerMinute
}

function getPositionWidth(startStr: string, stopStr: string): number {
  const start = new Date(startStr).getTime()
  const stop = new Date(stopStr).getTime()
  const diffMinutes = (stop - start) / 60000
  return diffMinutes * pxPerMinute
}

// Clamp EPG entry start to the visible timeline start to prevent rendering behind channel column
function getRenderStart(startStr: string): string {
  const start = new Date(startStr).getTime()
  const timelineStart = startTime.value.getTime()
  if (start < timelineStart) {
    return startTime.value.toISOString()
  }
  return startStr
}

// Current time marker position
const currentTimeLeft = computed(() => {
  const diffMinutes = (now.value.getTime() - startTime.value.getTime()) / 60000
  return diffMinutes * pxPerMinute
})

// Scrolling behavior
const scrollContainer = ref<HTMLElement | null>(null)
let lastScrollLeft = 0

function scrollToNow() {
  if (scrollContainer.value) {
    const containerWidth = scrollContainer.value.clientWidth
    const channelColumnWidth = 200 // sidebar width
    const targetScroll = currentTimeLeft.value - (containerWidth - channelColumnWidth) / 2
    scrollContainer.value.scrollLeft = Math.max(0, targetScroll)
    lastScrollLeft = scrollContainer.value.scrollLeft
    isTimeCentered.value = true
  }
}

// Timeline Pagination Limits (24h back / forward)
const canShiftBackward = computed(() => {
  const minTime = new Date().getTime() - 24 * 3600 * 1000
  // Check if shifting back 6h stays above -24h limit
  return baseTime.value.getTime() - 6 * 3600 * 1000 >= minTime
})

const canShiftForward = computed(() => {
  const maxTime = new Date().getTime() + 24 * 3600 * 1000
  // Check if shifting forward 6h stays below +24h limit
  return baseTime.value.getTime() + 6 * 3600 * 1000 <= maxTime
})

function shiftTime(hours: number) {
  const minTime = new Date().getTime() - 24 * 3600 * 1000
  const maxTime = new Date().getTime() + 24 * 3600 * 1000
  const targetTime = baseTime.value.getTime() + hours * 3600 * 1000
  
  if (targetTime >= minTime && targetTime <= maxTime) {
    baseTime.value = new Date(targetTime)
  }
}

// Jump back to current time
function goToday() {
  baseTime.value = new Date()
  nextTick(() => {
    scrollToNow()
  })
}

function handleScroll() {
  if (!scrollContainer.value || !isTimeCentered.value) return
  
  const container = scrollContainer.value
  const scrollLeft = container.scrollLeft
  if (scrollLeft === lastScrollLeft) return
  lastScrollLeft = scrollLeft
}

const isDesktop = computed(() => {
  return checkIsDesktop()
})
const rowHeight = computed(() => isDesktop.value ? 72 : 50)
const rowHeightPx = computed(() => `${rowHeight.value}px`)

const rowVirtualizer = useVirtualizer(
  computed(() => ({
    count: filteredChannels.value?.length || 0,
    getScrollElement: () => scrollContainer.value,
    estimateSize: () => rowHeight.value,
    overscan: 10,
  }))
)

// Watch for data changes and load the next page in the background automatically
// Uses requestAnimationFrame yielding to prevent CPU starvation and allow smooth UI transitions
watch(() => infiniteData.value, async () => {
  if (hasNextPage.value && !isFetchingNextPage.value) {
    await new Promise((resolve) => requestAnimationFrame(resolve))
    fetchNextPage()
  }
})

// Watch for scrollContainer to appear (after loading finishes) and center EPG timeline
watch(scrollContainer, (newVal) => {
  if (newVal && !isTimeCentered.value) {
    nextTick(() => {
      scrollToNow()
    })
  }
})
function shouldShowEndTime(startStr: string, stopStr: string): boolean {
  try {
    const start = new Date(startStr).getTime()
    const stop = new Date(stopStr).getTime()
    const durationMin = (stop - start) / 60000
    return durationMin > 30
  } catch (e) {
    return true
  }
}

// Format helpers
function formatTime(date: Date): string {
  const h = String(date.getHours()).padStart(2, '0')
  const m = String(date.getMinutes()).padStart(2, '0')
  return `${h}:${m}`
}

function formatEpgTime(dateStr: string): string {
  try {
    const date = new Date(dateStr)
    return formatTime(date)
  } catch (e) {
    return ''
  }
}

// Program status checks
const isPastProgram = (stopStr: string) => checkPastProgram(stopStr, now.value)
const isCurrentProgram = (startStr: string, stopStr: string) => checkCurrentProgram(startStr, stopStr, now.value)

// Selected program / channel for details sidebar
const selectedChannel = ref<any | null>(null)
const selectedProgram = ref<any | null>(null)
const isMobileDetailOpen = ref(false)

function selectProgram(channel: any, program: any) {
  selectedChannel.value = channel
  selectedProgram.value = program
  isMobileDetailOpen.value = true
}

function closeDetails() {
  selectedChannel.value = null
  selectedProgram.value = null
  isMobileDetailOpen.value = false
}

const showSidebarActions = computed(() => {
  if (!selectedProgram.value || !selectedChannel.value) return false
  
  if (isCurrentProgram(selectedProgram.value.start, selectedProgram.value.stop)) {
    return true
  }
  
  if (isPastProgram(selectedProgram.value.stop)) {
    return selectedChannel.value.tv_archive === 1
  }
  
  return false
})

const isPlayDisabled = computed(() => {
  if (!selectedProgram.value || !selectedChannel.value) return true
  if (isPastProgram(selectedProgram.value.stop)) {
    return !isCatchupValid.value
  }
  return false
})

// Playback handlers
function handlePlay() {
  if (!selectedChannel.value || !selectedProgram.value) return
  if (isCurrentProgram(selectedProgram.value.start, selectedProgram.value.stop)) {
    playLive(selectedChannel.value.stream_id, selectedChannel.value.profile_id)
  } else if (isPastProgram(selectedProgram.value.stop) && selectedChannel.value.tv_archive === 1) {
    const startDateTime = formatUtcForCatchup(selectedProgram.value.start, selectedProgram.value.tz_offset)
    const duration = getDurationMinutes(selectedProgram.value.start, selectedProgram.value.stop)
    playCatchup(selectedChannel.value.stream_id, startDateTime, duration, selectedChannel.value.profile_id)
  }
}

async function copyUrl() {
  if (!selectedChannel.value || !selectedProgram.value) return
  try {
    const profile = profileStore.profiles.find(p => p.id === selectedChannel.value.profile_id)
    if (!profile) return

    if (profile.profile_type === 'public_iptv') {
      const { resolveStreamUrl, copyToSystemClipboard } = await import('@/lib/tauri-commands')
      const tempUrl = `https://public/live/${selectedChannel.value.stream_id}.ts`
      const url = await resolveStreamUrl(tempUrl, profile.id)
      await copyToSystemClipboard(url)
      toastStore.showToast(t('media.urlCopied'), 'success')
      return
    }

    const password = profile.password
    if (!password) return

    let url = ''
    if (isCurrentProgram(selectedProgram.value.start, selectedProgram.value.stop)) {
      url = buildLiveUrl({
        serverUrl: profile.server_url,
        username: profile.username,
        password,
      }, selectedChannel.value.stream_id, 'ts')
    } else if (isPastProgram(selectedProgram.value.stop) && selectedChannel.value.tv_archive === 1) {
      const startDateTime = formatUtcForCatchup(selectedProgram.value.start, selectedProgram.value.tz_offset)
      const duration = getDurationMinutes(selectedProgram.value.start, selectedProgram.value.stop)
      url = buildCatchupUrl({
        serverUrl: profile.server_url,
        username: profile.username,
        password,
      }, selectedChannel.value.stream_id, startDateTime, duration)
    }

    if (url) {
      const { copyToSystemClipboard } = await import('@/lib/tauri-commands')
      await copyToSystemClipboard(url)
      toastStore.showToast(t('media.urlCopied'), 'success')
    }
  } catch (err) {
    console.error(err)
    toastStore.showToast(t('media.copyFailed'), 'error')
  }
}

// Watch selected channel and program for catchup pre-check
const isCatchupValid = ref(false)
let lastCheckStreamId: number | null = null

watch([selectedChannel, selectedProgram], async ([newChannel, newProgram]) => {
  isCatchupValid.value = false
  if (!newChannel || newChannel.tv_archive !== 1 || !newProgram || !isPastProgram(newProgram.stop)) {
    lastCheckStreamId = null
    return
  }

  const streamId = newChannel.stream_id
  lastCheckStreamId = streamId

  try {
    const profile = profileStore.profiles.find(p => p.id === newChannel.profile_id)
    if (!profile) return

    const password = profile.password
    if (!password) return

    const startDateTime = formatUtcForCatchup(newProgram.start, newProgram.tz_offset)
    const duration = getDurationMinutes(newProgram.start, newProgram.stop)

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
    const valid = await validateStreamUrl(url, newChannel.profile_id)

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
})
</script>

<template>
  <div class="guide-view-container">
    <!-- EPG Header Area -->
    <header class="guide-header-bar">
      <div class="header-left">
        <h2 class="page-title">{{ $t('sidebar.guide') }}</h2>
        <div class="timeline-nav-buttons">
          <button class="nav-btn" :disabled="!canShiftBackward" @click="shiftTime(-6)">
            <svg viewBox="0 0 24 24" class="btn-icon" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="15 18 9 12 15 6"></polyline>
            </svg>
            <span>-6h</span>
          </button>
          <button class="nav-btn" :disabled="!canShiftForward" @click="shiftTime(6)">
            <span>+6h</span>
            <svg viewBox="0 0 24 24" class="btn-icon" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
          </button>
          <button class="nav-btn today-btn" @click="goToday">
            {{ $t('media.today')}}
          </button>
        </div>
      </div>
      <div class="header-right-actions">
        <CustomSelect
          v-model="selectedProfileId"
          :options="profileOptions"
          style="width: 180px; flex-shrink: 0;"
        />
        <CustomSelect
          v-model="selectedCountry"
          :options="countryOptions"
          style="width: 160px; flex-shrink: 0;"
        />
        <label class="custom-checkbox">
          <input type="checkbox" v-model="showCatchupOnly" class="checkbox-input" />
          <span class="checkbox-box">
            <IconCheck class="checkbox-check" />
          </span>
          <span class="checkbox-label">{{ $t('media.catchupOnly') }}</span>
        </label>
        <input
          type="text"
          v-model="searchQuery"
          :placeholder="$t('sidebar.placeholder')"
          class="guide-search-input"
        />
      </div>
    </header>

    <!-- Loading Screen -->
    <div v-if="isLoading && (!channels || channels.length === 0)" class="guide-loading">
      <span class="spinner"></span>
      <p>{{ $t('settings.stats.loading') }}</p>
    </div>

    <div v-else-if="error" class="guide-error">
      <p>{{ $t('settings.sync.failed', { error: error.message || String(error) }) }}</p>
    </div>

    <!-- Empty State Screen -->
    <div v-else-if="!cleanChannels || cleanChannels.length === 0" class="guide-empty">
      <p>{{ $t('media.guideNoData') }}</p>
    </div>

    <!-- Main TV Guide Workspace -->
    <div v-else class="guide-main-workspace" :class="{ 'compact-rows': !isDesktop }">
      <!-- Scrollable EPG Timeline Grid -->
      <div class="guide-scroll-container" ref="scrollContainer" @scroll="handleScroll">
        <div class="guide-grid-wrapper" :style="{ width: `${totalTimelineMinutes * pxPerMinute + 200}px`, height: `${rowVirtualizer.getTotalSize() + 36}px` }">
          
          <!-- Sticky Headers Row -->
          <div class="guide-sticky-header">
            <div class="channel-header-filler"></div>
            <div class="timeline-header">
              <div
                v-for="slot in timeSlots"
                :key="slot.toISOString()"
                v-show="getPositionLeft(slot.toISOString()) >= 0"
                class="time-slot-tick"
                :style="{ left: `${getPositionLeft(slot.toISOString())}px` }"
              >
                {{ formatTime(slot) }}
              </div>
              <!-- Red Knob on Timeline Header -->
              <div v-show="currentTimeLeft >= 0" class="current-time-indicator" :style="{ left: `${currentTimeLeft}px` }">
                <div class="indicator-knob"></div>
              </div>
            </div>
          </div>

          <!-- Vertical Current Time Line Pointer extending down the grid -->
          <div v-show="currentTimeLeft >= 0" class="current-time-line-indicator" :style="{ left: `${currentTimeLeft + 200}px`, height: `${rowVirtualizer.getTotalSize() + 36}px` }"></div>

          <!-- Virtualized Rows Container -->
          <div class="virtual-rows-container" :style="{ height: `${rowVirtualizer.getTotalSize()}px` }">
            <div
              v-for="virtualRow in rowVirtualizer.getVirtualItems()"
              :key="virtualRow.index"
              class="virtual-row-item"
              :style="{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${virtualRow.size}px`,
                transform: `translateY(${virtualRow.start}px)`,
              }"
            >
              <!-- Left sticky channel column cell -->
              <div class="channel-cell-sticky">
                <div class="channel-logo">
                  <CachedImage
                    :src="filteredChannels[virtualRow.index].stream_icon"
                    :alt="filteredChannels[virtualRow.index].name || ''"
                    :fallback-text="filteredChannels[virtualRow.index].name || ''"
                  />
                </div>
                <span class="channel-name" :title="filteredChannels[virtualRow.index].name || ''">
                  {{ filteredChannels[virtualRow.index].name }}
                </span>
              </div>

              <!-- Right timeline schedule content row -->
              <div class="grid-schedule-row">
                <div
                  v-for="entry in filteredChannels[virtualRow.index].epg_entries"
                  :key="entry.start"
                  class="program-block"
                  :class="{
                    'past-prog': isPastProgram(entry.stop),
                    'current-prog': isCurrentProgram(entry.start, entry.stop),
                    'active': selectedProgram?.start === entry.start && selectedChannel?.stream_id === filteredChannels[virtualRow.index].stream_id
                  }"
                  :style="{
                    left: `${getPositionLeft(getRenderStart(entry.start))}px`,
                    width: `${getPositionWidth(getRenderStart(entry.start), entry.stop)}px`
                  }"
                  @click="selectProgram(filteredChannels[virtualRow.index], entry)"
                >
                  <div class="program-block-inner">
                    <div class="program-title-line">
                      <span v-if="isCurrentProgram(entry.start, entry.stop)" class="live-tag">{{ $t('media.liveTag') }}</span>
                      <span class="program-title" :title="entry.title || ''">{{ entry.title }}</span>
                    </div>
                    <span class="program-time">
                      <template v-if="shouldShowEndTime(entry.start, entry.stop)">
                        {{ formatEpgTime(entry.start) }} - {{ formatEpgTime(entry.stop) }}
                      </template>
                      <template v-else>
                        {{ formatEpgTime(entry.start) }}
                      </template>
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Side details sidebar -->
      <LiveDetailPanel
        :stream="selectedChannel"
        :is-mobile-open="isMobileDetailOpen"
        :show-actions="showSidebarActions"
        :play-disabled="isPlayDisabled"
        @close="closeDetails"
        @play="handlePlay"
        @copy="copyUrl"
        @toggle-favorite="handleToggleFavorite"
      >
        <!-- Custom selected program detail block in EPG Sidebar -->
        <div v-if="selectedProgram" class="guide-selection-details">
          <h4 class="selection-title">{{ selectedProgram.title }}</h4>
          <span class="selection-time">
            {{ formatEpgTime(selectedProgram.start) }} - {{ formatEpgTime(selectedProgram.stop) }}
            ({{ getDurationMinutes(selectedProgram.start, selectedProgram.stop) }} min)
          </span>
          <p v-if="selectedProgram.description" class="selection-desc">{{ selectedProgram.description }}</p>
        </div>
      </LiveDetailPanel>
    </div>
  </div>
</template>

<style scoped>
.guide-view-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  overflow: hidden;
  background-color: var(--color-bg);
}

.guide-main-workspace {
  display: flex;
  flex-direction: row;
  flex-grow: 1;
  height: 0;
  overflow: hidden;
}

.guide-header-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-2) var(--spacing-6);
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-surface);
  flex-shrink: 0;
}

.header-right-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-6);
}

.guide-search-input {
  background-color: var(--color-surface-hover);
  border: 1px solid var(--color-border);
  color: var(--color-text);
  padding: var(--spacing-2) var(--spacing-4);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
  width: 280px;
  outline: none;
  transition: all var(--transition-fast) ease;
}

.guide-search-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--spacing-6);
}

.page-title {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-text);
}

.timeline-nav-buttons {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
}

.nav-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-1);
  background-color: var(--color-surface-hover, #f3f4f6);
  border: 1px solid var(--color-border);
  color: var(--color-text);
  border-radius: var(--radius-md);
  padding: var(--spacing-2) var(--spacing-3);
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  outline: none;
  transition: all var(--transition-fast) ease;
}

[data-theme='dark'] .nav-btn {
  background-color: var(--color-surface);
}

.nav-btn:hover:not(:disabled) {
  background-color: var(--color-border);
  border-color: var(--color-text-muted);
}

.nav-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-icon {
  width: 12px;
  height: 12px;
}

.today-btn {
  background-color: var(--color-primary);
  color: #fff;
  border-color: var(--color-primary);
}

.today-btn:hover:not(:disabled) {
  background-color: var(--color-primary-hover);
  border-color: var(--color-primary-hover);
}

.guide-sub {
  font-size: 0.85rem;
  color: var(--color-text-muted);
}

.now-button {
  background-color: var(--color-primary);
  color: #fff;
  border: none;
  border-radius: var(--radius-md);
  padding: var(--spacing-2) var(--spacing-4);
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  transition: background-color var(--transition-fast) ease;
}

.now-button:hover {
  background-color: var(--color-primary-hover);
}

/* Loading & Empty state */
.guide-loading, .guide-error, .guide-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex-grow: 1;
  gap: var(--spacing-4);
  color: var(--color-text-muted);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Horizontal timeline layout */
.guide-scroll-container {
  flex-grow: 1;
  overflow: auto;
  position: relative;
}

.guide-grid-wrapper {
  display: flex;
  position: relative;
  height: 100%;
}

/* Sticky Header (Vertical & Horizontal Scroll Sync) */
.guide-sticky-header {
  display: flex;
  position: sticky;
  top: 0;
  height: 36px;
  z-index: 15;
  background-color: var(--color-surface);
  width: 100%;
  flex-shrink: 0;
}

.channel-header-filler {
  width: 200px;
  height: 36px;
  position: sticky;
  left: 0;
  z-index: 16;
  background-color: var(--color-surface);
  border-right: 1px solid var(--color-border);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.channel-logo {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-sm);
  overflow: hidden;
  flex-shrink: 0;
}

.channel-name {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Time slots header */
.timeline-header {
  flex-grow: 1;
  height: 36px;
  position: relative;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-surface);
  z-index: 5;
}

.time-slot-tick {
  position: absolute;
  top: 9px;
  transform: translateX(-50%);
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text-muted);
}

/* Current time background pointer line extending down the grid */
.current-time-line-indicator {
  position: absolute;
  top: 0;
  width: 1px;
  background-color: rgba(239, 68, 68, 0.4);
  z-index: 4;
  pointer-events: none;
}

/* Virtualized list styles */
.virtual-rows-container {
  position: absolute;
  top: 36px;
  left: 0;
  width: 100%;
}

.virtual-row-item {
  display: flex;
  height: v-bind(rowHeightPx);
}

.channel-cell-sticky {
  width: 200px;
  height: v-bind(rowHeightPx);
  position: sticky;
  left: 0;
  z-index: 10;
  background-color: var(--color-surface);
  border-right: 1px solid var(--color-border);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  padding: 0 var(--spacing-4);
  gap: var(--spacing-3);
  flex-shrink: 0;
}

.compact-rows .channel-cell-sticky {
  padding: 0 var(--spacing-3);
  gap: var(--spacing-2);
}

.grid-schedule-row {
  flex-grow: 1;
  height: v-bind(rowHeightPx);
  position: relative;
  border-bottom: 1px solid var(--color-border);
}

/* Program Blocks */
.program-block {
  position: absolute;
  top: var(--spacing-2);
  bottom: var(--spacing-2);
  border-radius: var(--radius-md);
  padding: var(--spacing-2) var(--spacing-3);
  cursor: pointer;
  box-sizing: border-box;
  background-color: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--color-border);
  transition: all var(--transition-fast);
  overflow: hidden;
}

.compact-rows .program-block {
  top: var(--spacing-1);
  bottom: var(--spacing-1);
  padding: 2px var(--spacing-2);
}

.program-block:hover {
  background-color: rgba(255, 255, 255, 0.08);
  border-color: var(--color-primary);
  transform: scale(1.01);
}

.program-block.active {
  background-color: rgba(96, 165, 250, 0.15);
  border-color: var(--color-primary);
  box-shadow: 0 0 12px rgba(96, 165, 250, 0.2);
}

/* Differentiate timelines visually */
.program-block.past-prog {
  opacity: 0.65;
  background-color: rgba(255, 255, 255, 0.01);
}

.program-block.current-prog {
  background-color: rgba(59, 130, 246, 0.06);
  border-color: rgba(59, 130, 246, 0.35);
}

.program-block-inner {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  height: 100%;
}

.program-title-line {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  overflow: hidden;
}

.live-tag {
  background-color: #ef4444;
  color: #fff;
  font-size: 0.65rem;
  font-weight: 800;
  padding: 1px 4px;
  border-radius: var(--radius-sm);
  letter-spacing: 0.05em;
  flex-shrink: 0;
}

.program-title {
  font-size: 0.85rem;
  font-weight: 700;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.compact-rows .program-title {
  font-size: 0.8rem;
}

.program-time {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.compact-rows .program-time {
  font-size: 0.7rem;
}

/* Time Indicator Line */
.current-time-indicator {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 2px;
  background-color: #ef4444;
  z-index: 8;
  pointer-events: none;
}

.indicator-knob {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #ef4444;
  position: absolute;
  top: 0;
  left: -3px;
  box-shadow: 0 0 8px #ef4444;
}

/* Selection detail block styles */
.guide-selection-details {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
  padding: var(--spacing-4) 0;
}

.selection-title {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--color-text);
}

.selection-time {
  font-size: 0.85rem;
  color: var(--color-text-muted);
  font-weight: 600;
}

.selection-desc {
  font-size: 0.92rem;
  color: var(--color-text-muted);
  line-height: 1.6;
}

.catchup-actions-box {
  margin-top: var(--spacing-4);
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-6);
}

.catchup-enabled-container {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.catchup-disabled-notice {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  color: var(--color-text-muted);
  font-size: 0.85rem;
  background-color: rgba(255, 255, 255, 0.03);
  padding: var(--spacing-3);
  border-radius: var(--radius-md);
  border: 1px dashed var(--color-border);
}

.notice-icon {
  width: 16px;
  height: 16px;
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

</style>
