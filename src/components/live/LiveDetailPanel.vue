<script setup lang="ts">
import { ref, computed, onUnmounted, watch } from 'vue'
import StreamDetailPanel from '@/components/ui/StreamDetailPanel.vue'
import IconStar from '@/components/icons/IconStar.vue'
import IconPlay from '@/components/icons/IconPlay.vue'
import IconCopy from '@/components/icons/IconCopy.vue'
import IconLive from '@/components/icons/IconLive.vue'
import { useEpg } from '@/composables/useEpg'
import { useProfileStore } from '@/stores/profile.store'
import { useToastStore } from '@/stores/toast.store'
import { getSetting, copyToSystemClipboard } from '@/lib/tauri-commands'
import { usePlayer } from '@/composables/usePlayer'
import { buildCatchupUrl } from '@/lib/url-builder'
import { useI18n } from '@/composables/useI18n'
import type { LiveStream } from '@/types/stream'
import {
  formatUtcForCatchup,
  getDurationMinutes,
  isCurrentProgram as checkCurrentProgram,
  isPastProgram as checkPastProgram
} from '@/lib/date-utils'

const props = withDefaults(defineProps<{
  stream: LiveStream | null
  isMobileOpen: boolean
  showActions?: boolean
  playDisabled?: boolean
}>(), {
  showActions: true,
  playDisabled: false
})

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'play', stream: LiveStream): void
  (e: 'copy', stream: LiveStream): void
  (e: 'toggle-favorite', stream: LiveStream): void
}>()

const { t } = useI18n()
const profileStore = useProfileStore()
const toastStore = useToastStore()
const { playCatchup } = usePlayer()

// ─── EPG guide setup ─────────────────────────────────────────────────────────
const selectedEpgChannelId = computed(() => props.stream?.epg_channel_id || null)

const hoursBack = computed(() => {
  if (props.stream && props.stream.tv_archive === 1) {
    return (props.stream.tv_archive_duration || 1) * 24
  }
  return 1
})

const { data: epgData, isLoading: isLoadingEpg } = useEpg(
  computed(() => props.stream?.profile_id),
  selectedEpgChannelId,
  hoursBack
)

const now = ref(new Date())
const timer = setInterval(() => {
  now.value = new Date()
}, 30000)

onUnmounted(() => {
  clearInterval(timer)
})

const isPastProgram = (stopStr: string) => checkPastProgram(stopStr, now.value)
const isCurrentProgram = (startStr: string, stopStr: string) => checkCurrentProgram(startStr, stopStr, now.value)

const currentProgram = computed(() => {
  if (!epgData.value) return null
  return epgData.value.find((entry) => isCurrentProgram(entry.start, entry.stop)) || null
})

const pastPrograms = computed(() => {
  if (!epgData.value || !props.stream || props.stream.tv_archive !== 1) return []
  const durationMs = (props.stream.tv_archive_duration || 0) * 24 * 3_600_000
  const cutoffTime = new Date(now.value.getTime() - durationMs)
  
  const finished = epgData.value.filter((entry) => {
    const start = new Date(entry.start)
    return isPastProgram(entry.stop) && start >= cutoffTime
  })
  
  return [...finished].sort((a, b) => new Date(b.start).getTime() - new Date(a.start).getTime())
})

const isCatchupValid = ref(false)
let lastCheckStreamId: number | null = null

watch(() => props.stream, () => {
  isCatchupValid.value = false
  lastCheckStreamId = null
})

watch(pastPrograms, async (newPastPrograms) => {
  if (!newPastPrograms || newPastPrograms.length === 0 || !props.stream) {
    isCatchupValid.value = false
    return
  }

  if (isCatchupValid.value && lastCheckStreamId === props.stream.stream_id) {
    return
  }

  const streamId = props.stream.stream_id
  lastCheckStreamId = streamId
  isCatchupValid.value = false

  try {
    const profile = profileStore.profile
    if (!profile) return

    const password = await getSetting('password')
    if (!password) return

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
    const valid = await validateStreamUrl(url, props.stream.profile_id)

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
  if (!props.stream) return
  const startDateTime = formatUtcForCatchup(item.start, item.tz_offset)
  const duration = getDurationMinutes(item.start, item.stop)
  await playCatchup(props.stream.stream_id, startDateTime, duration, props.stream.profile_id)
}

async function copyCatchupUrl(item: any) {
  try {
    const profile = profileStore.profile
    if (!profile || !props.stream) {
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
      props.stream.stream_id,
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
  <StreamDetailPanel
    :stream="stream"
    :is-mobile-open="isMobileOpen"
    square-image
    :default-width="500"
    :play-button-text="$t('media.play')"
    :copy-button-text="$t('media.copyUrl')"
    :show-actions="showActions"
    :play-disabled="playDisabled"
    @close="emit('close')"
    @play="emit('play', stream!)"
    @copy="emit('copy', stream!)"
  >
    <template #header-meta>
      <div class="live-header-meta-row">
        <span v-if="stream?.tv_archive === 1" class="archive-text">
          ⏱ {{ $t('media.catchupDays', { days: stream.tv_archive_duration }) }}
        </span>
        <button class="btn-fav" :class="{ favorited: !!stream?.is_favorite }" @click="emit('toggle-favorite', stream!)">
          <IconStar class="fav-icon" />
          <span>{{ stream?.is_favorite ? 'Favorited' : 'Favorite' }}</span>
        </button>
      </div>
    </template>

    <template #header-meta-mobile>
      <div class="live-header-meta-row">
        <span v-if="stream?.tv_archive === 1" class="archive-text">
          ⏱ {{ $t('media.catchupDays', { days: stream.tv_archive_duration }) }}
        </span>
        <button class="btn-fav" :class="{ favorited: !!stream?.is_favorite }" @click="emit('toggle-favorite', stream!)">
          <IconStar class="fav-icon" />
          <span>{{ stream?.is_favorite ? 'Favorited' : 'Favorite' }}</span>
        </button>
      </div>
    </template>

    <!-- Custom layout slot if parent view overrides details layout (e.g. GuideView) -->
    <slot>
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
          <div v-if="stream?.tv_archive === 1 && pastPrograms.length > 0" class="epg-past-section">
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
    </slot>

    <template #no-selection>
      <div class="no-selection">
        <div class="tv-art">
          <IconLive />
        </div>
        <p>{{ $t('media.selectToView') }}</p>
      </div>
    </template>
  </StreamDetailPanel>
</template>

<style scoped>
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

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.spinner.small {
  width: 16px;
  height: 16px;
  border-width: 2px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
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
