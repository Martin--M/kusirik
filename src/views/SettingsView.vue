<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useProfileStore, PROFILE_ID } from '@/stores/profile.store'
import { useSettingsStore } from '@/stores/settings.store'
import { useSyncStore } from '@/stores/sync.store'
import { useToastStore } from '@/stores/toast.store'
import { 
  deleteProfile, 
  getSyncStatus, 
  triggerSync,
  getLiveCategories,
  getVodCategories,
  getSeriesCategories
} from '@/lib/tauri-commands'
import CustomSelect from '@/components/ui/CustomSelect.vue'
import type { DataType } from '@/types/sync'
import IconProfile from '@/components/icons/IconProfile.vue'
import IconLogOut from '@/components/icons/IconLogOut.vue'
import IconStats from '@/components/icons/IconStats.vue'
import IconLive from '@/components/icons/IconLive.vue'
import IconMovies from '@/components/icons/IconMovies.vue'
import IconSeries from '@/components/icons/IconSeries.vue'
import IconCalendar from '@/components/icons/IconCalendar.vue'
import IconSettings from '@/components/icons/IconSettings.vue'
import IconMoon from '@/components/icons/IconMoon.vue'
import IconSun from '@/components/icons/IconSun.vue'
import IconPlay from '@/components/icons/IconPlay.vue'
import IconSync from '@/components/icons/IconSync.vue'

const router = useRouter()
const profileStore = useProfileStore()
const settingsStore = useSettingsStore()
const syncStore = useSyncStore()
const toastStore = useToastStore()

const liveCatCount = ref<number | null>(null)
const vodCatCount = ref<number | null>(null)
const seriesCatCount = ref<number | null>(null)

const isAndroid = ref(false)
const dataTypes: DataType[] = ['live_streams', 'vod_streams', 'series', 'epg']

const formatOptions = {
  "": "Default (Auto-detect)",
  "ts": "MPEG-TS (.ts)",
  "m3u8": "HLS (.m3u8)"
}

const selectedFormat = computed({
  get: () => settingsStore.liveFormatOverride || "",
  set: (val) => {
    settingsStore.setLiveFormatOverride(val === "" ? null : val)
    toastStore.showToast(`Format override updated to: ${val || 'Default'}`, 'success')
  }
})

// Input local bindings to avoid firing DB write on every keystroke
const playerWindowsInput = ref('')
const playerAndroidInput = ref('')

watch(() => settingsStore.playerWindows, (newVal) => {
  playerWindowsInput.value = newVal
})

watch(() => settingsStore.playerAndroid, (newVal) => {
  playerAndroidInput.value = newVal
})

async function savePlayerWindows() {
  const val = playerWindowsInput.value.trim()
  if (val !== settingsStore.playerWindows) {
    try {
      await settingsStore.setPlayerWindows(val)
      toastStore.showToast('Windows player path saved', 'success')
    } catch (e) {
      toastStore.showToast('Failed to save Windows player path', 'error')
    }
  }
}

async function savePlayerAndroid() {
  const val = playerAndroidInput.value.trim()
  if (val !== settingsStore.playerAndroid) {
    try {
      await settingsStore.setPlayerAndroid(val)
      toastStore.showToast('Android package ID saved', 'success')
    } catch (e) {
      toastStore.showToast('Failed to save Android package ID', 'error')
    }
  }
}

const isGlobalSyncing = computed(() => {
  return Object.values(syncStore.statuses).some((s) => s.is_syncing)
})

async function handleSyncAll() {
  if (isGlobalSyncing.value) return
  
  const confirmed = confirm(
    'Are you sure you want to synchronize the database? This will update category listings and streams cache from your provider. This process runs in the background.'
  )
  if (!confirmed) return

  toastStore.showToast('Starting database synchronization...', 'success')
  try {
    await triggerSync('live_streams')
  } catch (err) {
    toastStore.showToast(`Sync failed: ${err}`, 'error')
  }
}

async function handleDisconnect() {
  const confirmed = confirm(
    'Are you sure you want to disconnect? This will remove your configuration profile and clear all cached media streams from the local database.'
  )
  if (!confirmed) return

  try {
    toastStore.showToast('Disconnecting profile...', 'success')
    await deleteProfile(PROFILE_ID)
    profileStore.clearProfile()
    syncStore.reset()
    toastStore.showToast('Profile disconnected successfully', 'success')
    router.push('/setup')
  } catch (err) {
    toastStore.showToast(`Failed to disconnect profile: ${err}`, 'error')
  }
}

function formatTime(isoString: string | null): string {
  if (!isoString) return 'Never'
  try {
    const date = new Date(isoString)
    const y = date.getFullYear()
    const m = String(date.getMonth() + 1).padStart(2, '0')
    const d = String(date.getDate()).padStart(2, '0')
    const hr = String(date.getHours()).padStart(2, '0')
    const min = String(date.getMinutes()).padStart(2, '0')
    return `${y}-${m}-${d} ${hr}:${min}`
  } catch (e) {
    return isoString
  }
}

function getTypeName(type: DataType): string {
  switch (type) {
    case 'live_streams': return 'Live TV Channels'
    case 'vod_streams': return 'VOD Movies'
    case 'series': return 'TV Series'
    case 'epg': return 'EPG Guide Data'
  }
}

// getTypeIcon removed in favor of inline SVGs

async function loadCounts() {
  try {
    const [liveCats, vodCats, seriesCats] = await Promise.all([
      getLiveCategories(PROFILE_ID).catch(() => []),
      getVodCategories(PROFILE_ID).catch(() => []),
      getSeriesCategories(PROFILE_ID).catch(() => [])
    ])
    liveCatCount.value = liveCats.length
    vodCatCount.value = vodCats.length
    seriesCatCount.value = seriesCats.length
  } catch (e) {
    console.error("Failed to load category counts:", e)
  }
}

onMounted(async () => {
  isAndroid.value = /Android/i.test(navigator.userAgent)
  await settingsStore.load()
  playerWindowsInput.value = settingsStore.playerWindows
  playerAndroidInput.value = settingsStore.playerAndroid

  try {
    const statuses = await getSyncStatus()
    for (const s of statuses) {
      if (s.fetched_at && s.item_count !== null) {
        syncStore.onDone(s.data_type as DataType, s.item_count, s.fetched_at)
      } else if (s.last_error) {
        syncStore.onError(s.data_type as DataType, s.last_error)
      }
    }
  } catch (e) {
    console.error("Failed to sync status logs on mount:", e)
  }

  await loadCounts()
})
</script>

<template>
  <div class="settings-view-container">
    <div class="settings-grid">
      <!-- Connection Profile Card -->
      <div class="settings-card profile-card">
        <div class="card-header">
          <IconProfile class="card-icon" />
          <h3>Connection Profile</h3>
        </div>
        <div class="card-content">
          <div class="info-group">
            <label>Server URL</label>
            <div class="info-value">{{ profileStore.profile?.server_url || 'N/A' }}</div>
          </div>
          <div class="info-group">
            <label>Username</label>
            <div class="info-value">{{ profileStore.profile?.username || 'N/A' }}</div>
          </div>
          <div class="card-actions">
            <button class="btn btn-danger" @click="handleDisconnect">
              <IconLogOut class="btn-icon" />
              Disconnect Profile
            </button>
          </div>
        </div>
      </div>

      <!-- Database Statistics Card -->
      <div class="settings-card stats-card">
        <div class="card-header">
          <IconStats class="card-icon" />
          <h3>Database Statistics</h3>
        </div>
        <div class="card-content">
          <div class="stats-list">
            <div class="stats-row">
              <div class="stats-row-header">
                <IconLive class="stats-svg" />
                <span class="stats-title">Live Channels</span>
              </div>
              <div class="stats-row-values">
                <span class="stats-badge">{{ liveCatCount !== null ? liveCatCount : '...' }} categories</span>
                <span class="stats-badge primary">{{ syncStore.statuses.live_streams?.item_count ?? 0 }} channels</span>
              </div>
            </div>
            <div class="stats-row">
              <div class="stats-row-header">
                <IconMovies class="stats-svg" />
                <span class="stats-title">VOD Movies</span>
              </div>
              <div class="stats-row-values">
                <span class="stats-badge">{{ vodCatCount !== null ? vodCatCount : '...' }} categories</span>
                <span class="stats-badge primary">{{ syncStore.statuses.vod_streams?.item_count ?? 0 }} movies</span>
              </div>
            </div>
            <div class="stats-row">
              <div class="stats-row-header">
                <IconSeries class="stats-svg" />
                <span class="stats-title">TV Series</span>
              </div>
              <div class="stats-row-values">
                <span class="stats-badge">{{ seriesCatCount !== null ? seriesCatCount : '...' }} categories</span>
                <span class="stats-badge primary">{{ syncStore.statuses.series?.item_count ?? 0 }} series</span>
              </div>
            </div>
            <div class="stats-row">
              <div class="stats-row-header">
                <IconCalendar class="stats-svg" />
                <span class="stats-title">EPG Guide Data</span>
              </div>
              <div class="stats-row-values">
                <span class="stats-badge primary">{{ syncStore.statuses.epg?.item_count ?? 0 }} entries</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- App Preferences Card -->
      <div class="settings-card preferences-card">
        <div class="card-header">
          <IconSettings class="card-icon" />
          <h3>Preferences</h3>
        </div>
        <div class="card-content">
          <div class="form-group">
            <label>Visual Theme</label>
            <div class="theme-toggle-group">
              <button 
                class="theme-btn" 
                :class="{ active: settingsStore.theme === 'dark' }"
                @click="settingsStore.setTheme('dark')"
              >
                <IconMoon />
                Dark
              </button>
              <button 
                class="theme-btn" 
                :class="{ active: settingsStore.theme === 'light' }"
                @click="settingsStore.setTheme('light')"
              >
                <IconSun />
                Light
              </button>
            </div>
          </div>

          <div class="form-group">
            <label>Live Stream Format Override</label>
            <CustomSelect
              v-model="selectedFormat"
              :options="formatOptions"
              ariaLabel="Select live stream format override"
            />
            <span class="subtext">
              Overrides the auto-selected container format for Live TV channels if supported by the provider.
            </span>
          </div>
        </div>
      </div>

      <!-- Media Player Configurations Card -->
      <div class="settings-card player-card">
        <div class="card-header">
          <IconPlay class="card-icon" />
          <h3>Media Player Configuration</h3>
        </div>
        <div class="card-content">
          <div v-if="!isAndroid" class="form-group">
            <label for="player-win-path">VLC Executable Path (Windows)</label>
            <input
              id="player-win-path"
              type="text"
              v-model="playerWindowsInput"
              @blur="savePlayerWindows"
              @keyup.enter="savePlayerWindows"
              placeholder="C:\Program Files\VideoLAN\VLC\vlc.exe"
              class="text-input"
            />
            <span class="subtext">
              Desktop player binary location. Leaving empty will fall back to default system protocol handlers.
            </span>
          </div>

          <div v-else class="form-group">
            <label for="player-android-pkg">External Player Package (Android)</label>
            <input
              id="player-android-pkg"
              type="text"
              v-model="playerAndroidInput"
              @blur="savePlayerAndroid"
              @keyup.enter="savePlayerAndroid"
              placeholder="org.videolan.vlc"
              class="text-input"
            />
            <span class="subtext">
              Target Android package name for opening streams (e.g. <code>org.videolan.vlc</code> or <code>com.mxtech.videoplayer.ad</code>). Leave blank to show standard Android chooser.
            </span>
          </div>
        </div>
      </div>

      <!-- Database Sync Logs Card -->
      <div class="settings-card sync-card">
        <div class="card-header">
          <IconSync class="card-icon" />
          <h3>Database Synchronization</h3>
        </div>
        <div class="card-content">
          <div class="table-container">
            <table class="sync-table">
              <thead>
                <tr>
                  <th>Category</th>
                  <th>Last Synced</th>
                  <th>Status</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="type in dataTypes" :key="type">
                  <td class="type-name-cell">
                    <div class="type-name-wrapper">
                      <IconLive v-if="type === 'live_streams'" class="stats-svg" />
                      <IconMovies v-else-if="type === 'vod_streams'" class="stats-svg" />
                      <IconSeries v-else-if="type === 'series'" class="stats-svg" />
                      <IconCalendar v-else-if="type === 'epg'" class="stats-svg" />
                      <span class="type-label">{{ getTypeName(type) }}</span>
                    </div>
                  </td>
                  <td class="type-time">{{ formatTime(syncStore.statuses[type]?.fetched_at) }}</td>
                  <td class="type-status">
                    <span v-if="syncStore.statuses[type]?.is_syncing" class="status-indicator syncing">
                      <span class="spinner mini"></span>
                      Syncing...
                    </span>
                    <span v-else-if="syncStore.statuses[type]?.last_error" class="status-indicator error" :title="syncStore.statuses[type]?.last_error!">
                      ⚠️ Error
                    </span>
                    <span v-else-if="syncStore.statuses[type]?.fetched_at" class="status-indicator success">
                      ✓ Ready
                    </span>
                    <span v-else class="status-indicator pending">
                      • Pending
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="sync-actions">
            <button 
              class="btn btn-primary btn-sync-all" 
              :disabled="isGlobalSyncing"
              @click="handleSyncAll"
            >
              <span v-if="isGlobalSyncing" class="spinner button-spinner"></span>
              {{ isGlobalSyncing ? 'Synchronizing Cache...' : 'Sync Database Now' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-view-container {
  padding: var(--spacing-6);
  height: 100%;
  overflow-y: auto;
  background-color: var(--color-bg);
}

.settings-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--spacing-6);
  max-width: 1200px;
  margin: 0 auto;
}

@media (min-width: 992px) {
  .settings-grid {
    grid-template-columns: 1fr 1fr;
  }
}

.settings-card {
  background: rgba(30, 41, 59, 0.45);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--spacing-6);
  box-shadow: var(--shadow-md);
  transition: transform var(--transition-fast), border-color var(--transition-fast), box-shadow var(--transition-fast);
}

[data-theme='light'] .settings-card {
  background: rgba(255, 255, 255, 0.65);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
}

.settings-card:hover {
  transform: translateY(-2px);
  border-color: var(--color-primary);
  box-shadow: var(--shadow-md), 0 0 15px rgba(96, 165, 250, 0.15);
}

[data-theme='light'] .settings-card:hover {
  box-shadow: 0 8px 30px rgba(59, 130, 246, 0.1);
}

.card-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  margin-bottom: var(--spacing-6);
  border-bottom: 1px solid var(--color-border);
  padding-bottom: var(--spacing-3);
}

.card-icon {
  width: 24px;
  height: 24px;
  color: var(--color-primary);
}

.card-header h3 {
  font-size: 1.15rem;
  font-weight: 700;
  color: var(--color-text);
  margin: 0;
}

.card-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
}

/* Info groups (Profile) */
.info-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-1);
}

.info-group label {
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted);
}

.info-value {
  font-size: 0.95rem;
  font-weight: 500;
  color: var(--color-text);
  word-break: break-all;
  padding: var(--spacing-2) var(--spacing-3);
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

[data-theme='light'] .info-value {
  background: rgba(0, 0, 0, 0.01);
}

/* Database Statistics styling */
.stats-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.stats-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-3) var(--spacing-4);
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

[data-theme='light'] .stats-row {
  background: rgba(0, 0, 0, 0.02);
}

.stats-row-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  font-weight: 600;
  color: var(--color-text);
}

.stats-icon {
  font-size: 1.1rem;
}

.stats-row-values {
  display: flex;
  gap: var(--spacing-2);
}

.stats-badge {
  font-size: 0.75rem;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: var(--radius-md);
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
}

[data-theme='light'] .stats-badge {
  background: rgba(0, 0, 0, 0.03);
}

.stats-badge.primary {
  background: rgba(96, 165, 250, 0.1);
  color: var(--color-primary);
  border-color: rgba(96, 165, 250, 0.2);
}

/* Inputs & Form Groups */
.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
}

.form-group label {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--color-text);
}

.text-input {
  width: 100%;
  padding: var(--spacing-3);
  background: rgba(15, 23, 42, 0.4);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-family: inherit;
  font-size: 0.9rem;
  outline: none;
  transition: all var(--transition-fast);
}

[data-theme='light'] .text-input {
  background: rgba(255, 255, 255, 0.8);
}

.text-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(96, 165, 250, 0.2);
}

/* Browse Button Layout */
.input-with-button {
  display: flex;
  gap: var(--spacing-2);
}

.input-with-button .text-input {
  flex: 1;
}

.btn-browse {
  white-space: nowrap;
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--color-border);
  color: var(--color-text);
  padding: 0 var(--spacing-4);
  height: 38px;
  font-size: 0.85rem;
}

[data-theme='light'] .btn-browse {
  background: rgba(0, 0, 0, 0.02);
}

.btn-browse:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: var(--color-text-muted);
}

[data-theme='light'] .btn-browse:hover {
  background: rgba(0, 0, 0, 0.04);
}

.subtext {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  line-height: 1.4;
}

.subtext code {
  background: rgba(255, 255, 255, 0.08);
  padding: 1px 4px;
  border-radius: var(--radius-sm);
  font-family: monospace;
}

[data-theme='light'] .subtext code {
  background: rgba(0, 0, 0, 0.05);
}

/* Theme Toggle button styles */
.theme-toggle-group {
  display: flex;
  gap: var(--spacing-3);
}

.theme-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-2);
  padding: var(--spacing-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background: rgba(255, 255, 255, 0.02);
  color: var(--color-text);
  font-family: inherit;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.theme-btn svg {
  width: 16px;
  height: 16px;
}

.theme-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: var(--color-text-muted);
}

[data-theme='light'] .theme-btn {
  background: rgba(0, 0, 0, 0.02);
}
[data-theme='light'] .theme-btn:hover {
  background: rgba(0, 0, 0, 0.04);
}

.theme-btn.active {
  background: linear-gradient(135deg, var(--color-primary) 0%, rgba(96, 165, 250, 0.6) 100%);
  border-color: var(--color-primary);
  color: white;
  box-shadow: 0 0 10px rgba(96, 165, 250, 0.3);
}

/* Table Design */
.table-container {
  overflow-x: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: rgba(0, 0, 0, 0.1);
}

[data-theme='light'] .table-container {
  background: rgba(255, 255, 255, 0.2);
}

.sync-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
  font-size: 0.85rem;
}

.sync-table th, .sync-table td {
  border-bottom: 1px solid var(--color-border);
}

.sync-table th {
  padding: var(--spacing-3) var(--spacing-4);
  font-weight: 700;
  color: var(--color-text-muted);
  text-transform: uppercase;
  font-size: 0.75rem;
  letter-spacing: 0.05em;
  background: rgba(255, 255, 255, 0.02);
}

.sync-table tbody tr:last-child td {
  border-bottom: none;
}

.type-name-cell {
  padding: var(--spacing-3) var(--spacing-4);
}

.type-name-wrapper {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  font-weight: 600;
  color: var(--color-text);
}

.type-icon {
  font-size: 1rem;
}

.type-time {
  padding: var(--spacing-3) var(--spacing-4);
  color: var(--color-text);
}

.type-status {
  padding: var(--spacing-3) var(--spacing-4);
}

/* Badges for sync status */
.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-1);
  font-size: 0.75rem;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 9999px;
}

.status-indicator.success {
  background: rgba(34, 197, 94, 0.15);
  color: #4ade80;
}

[data-theme='light'] .status-indicator.success {
  background: rgba(34, 197, 94, 0.15);
  color: #166534;
}

.status-indicator.error {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
}

[data-theme='light'] .status-indicator.error {
  background: rgba(239, 68, 68, 0.15);
  color: #991b1b;
}

.status-indicator.syncing {
  background: rgba(59, 130, 246, 0.15);
  color: #60a5fa;
}

.status-indicator.pending {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text-muted);
}

/* Spinner Animations */
.spinner {
  display: inline-block;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

.spinner.mini {
  width: 10px;
  height: 10px;
}

.spinner.button-spinner {
  width: 16px;
  height: 16px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Buttons styling */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-2);
  padding: var(--spacing-3) var(--spacing-6);
  border-radius: var(--radius-md);
  font-family: inherit;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  border: none;
  outline: none;
  transition: all var(--transition-fast);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: linear-gradient(135deg, var(--color-primary) 0%, rgba(96, 165, 250, 0.8) 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(96, 165, 250, 0.2);
}

.btn-primary:not(:disabled):hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(96, 165, 250, 0.3);
  background: linear-gradient(135deg, var(--color-primary-hover) 0%, var(--color-primary) 100%);
}

.btn-danger {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #f87171;
}

[data-theme='light'] .btn-danger {
  background: rgba(239, 68, 68, 0.1);
  color: #b91c1c;
}

.btn-danger:not(:disabled):hover {
  background: #ef4444;
  color: white;
  border-color: #ef4444;
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.2);
}

.btn-icon {
  width: 16px;
  height: 16px;
}

.sync-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: var(--spacing-4);
}

.btn-sync-all {
  width: 100%;
}

@media (min-width: 768px) {
  .btn-sync-all {
    width: auto;
  }
}

.stats-svg {
  width: 18px;
  height: 18px;
  color: var(--color-primary);
  flex-shrink: 0;
}
</style>
