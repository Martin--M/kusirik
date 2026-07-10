<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { testConnection, saveProfile, triggerSync } from '@/lib/tauri-commands'
import { useProfileStore } from '@/stores/profile.store'
import { useSyncStore } from '@/stores/sync.store'
import { useI18n } from '@/composables/useI18n'
import IconEye from '../components/icons/IconEye.vue'
import IconEyeOff from '../components/icons/IconEyeOff.vue'
import IconLogo from '../components/icons/IconLogo.vue'
const router = useRouter()
const route = useRoute()
const profileStore = useProfileStore()
const syncStore = useSyncStore()
const { t } = useI18n()

const serverUrl = ref('')
const username = ref('')
const password = ref('')
const showPassword = ref(false)

const isTesting = ref(false)
const testError = ref<string | null>(null)
const testSuccess = ref(false)

const isSaving = ref(false)
const saveError = ref<string | null>(null)

const isSyncing = ref(false)
const syncError = ref<string | null>(null)

const liveStatus = computed(() => syncStore.statuses.live_streams)
const vodStatus = computed(() => syncStore.statuses.vod_streams)
const seriesStatus = computed(() => syncStore.statuses.series)

const syncDone = computed(() => {
  return (
    liveStatus.value.fetched_at !== null &&
    vodStatus.value.fetched_at !== null &&
    seriesStatus.value.fetched_at !== null
  )
})

// Route to live TV once all lists have been synced successfully
watch(syncDone, (done) => {
  if (done && isSyncing.value) {
    isSyncing.value = false
    router.push('/live')
  }
})

// Capture errors during sync
watch(
  [
    () => liveStatus.value.last_error,
    () => vodStatus.value.last_error,
    () => seriesStatus.value.last_error,
  ],
  ([liveErr, vodErr, serErr]) => {
    if (liveErr || vodErr || serErr) {
      syncError.value = liveErr || vodErr || serErr
      isSyncing.value = false
    }
  }
)

async function handleTest() {
  if (!serverUrl.value || !username.value || !password.value) {
    testError.value = t('setup.validation')
    return
  }

  isTesting.value = true
  testError.value = null
  testSuccess.value = false

  try {
    await testConnection({
      url: serverUrl.value,
      username: username.value,
      password: password.value,
    })
    testSuccess.value = true
  } catch (err) {
    testError.value = String(err)
  } finally {
    isTesting.value = false
  }
}

async function saveProfileData() {
  const existingName = route.query.id
    ? (profileStore.profiles.find(p => p.id === Number(route.query.id))?.name || 'IPTV Provider')
    : 'IPTV Provider'

  const profile = await saveProfile({
    id: route.query.id ? Number(route.query.id) : undefined,
    name: existingName,
    server_url: serverUrl.value,
    username: username.value,
    password: password.value,
    epg_mode: 'xmltv',
  })

  profileStore.setProfile(profile)
  return profile
}

async function handleSave() {
  isSaving.value = true
  saveError.value = null

  try {
    await saveProfileData()
    isSaving.value = false
    router.push('/settings')
  } catch (err) {
    saveError.value = String(err)
    isSaving.value = false
  }
}

async function handleSaveAndSync() {
  isSaving.value = true
  saveError.value = null

  try {
    const profile = await saveProfileData()
    syncStore.reset()
    isSyncing.value = true
    syncError.value = null
    await triggerSync(profile.id, 'live_streams')
    isSaving.value = false
  } catch (err) {
    saveError.value = String(err)
    isSaving.value = false
  }
}

async function handleResync() {
  const confirmed = confirm(t('setup.resyncConfirm'))
  if (!confirmed) return

  isSaving.value = true
  saveError.value = null

  try {
    const profile = await saveProfileData()
    syncStore.reset()
    isSyncing.value = true
    syncError.value = null
    await triggerSync(profile.id, 'live_streams', true)
    isSaving.value = false
  } catch (err) {
    saveError.value = String(err)
    isSaving.value = false
  }
}

onMounted(async () => {
  initForm()
})

watch(() => route.fullPath, () => {
  initForm()
})

function initForm() {
  if (route.query.id) {
    const editId = Number(route.query.id)
    const existing = profileStore.profiles.find(p => p.id === editId)
    if (existing) {
      serverUrl.value = existing.server_url
      username.value = existing.username
      password.value = existing.password || ''
      testSuccess.value = true
    }
  } else {
    serverUrl.value = ''
    username.value = ''
    password.value = ''
    testSuccess.value = false
    if (route.name === 'setup' && profileStore.hasProfile && !route.query.add) {
      router.push('/')
    }
  }
}
</script>

<template>
  <div class="setup-container">
    <div class="glass-card">
      <div v-if="profileStore.hasProfile" class="back-nav" style="margin-bottom: var(--spacing-4);">
        <button type="button" class="btn btn-secondary btn-back" style="padding: var(--spacing-2) var(--spacing-4); font-size: 0.85rem;" @click="router.push('/settings')">
          ← {{ $t('setup.syncScreen.back') || 'Back' }}
        </button>
      </div>
      <div class="header">
        <IconLogo class="setup-logo-svg" />
        <h1 class="glow-title">{{ route.query.id ? $t('setup.editTitle') : 'kusirik' }}</h1>
        <p class="subtitle">{{ route.query.id ? $t('setup.editSubtitle') : $t('setup.subtitle') }}</p>
      </div>

      <!-- Main setup form -->
      <form v-if="!isSyncing" @submit.prevent class="setup-form">
        <div class="form-group">
          <label for="server-url">{{ $t('setup.serverUrl') }}</label>
          <input
            id="server-url"
            v-model="serverUrl"
            type="url"
            placeholder="http://example.com:8080"
            required
            :disabled="isTesting || isSaving"
          />
        </div>

        <div class="form-group">
          <label for="username">{{ $t('setup.username') }}</label>
          <input
            id="username"
            v-model="username"
            type="text"
            placeholder="Enter username"
            required
            :disabled="isTesting || isSaving"
          />
        </div>

        <div class="form-group">
          <label for="password">{{ $t('setup.password') }}</label>
          <div class="password-wrapper">
            <input
              id="password"
              v-model="password"
              :type="showPassword ? 'text' : 'password'"
              placeholder="Enter password"
              required
              :disabled="isTesting || isSaving"
            />
            <button
              type="button"
              class="toggle-password"
              @click="showPassword = !showPassword"
              tabindex="-1"
              :title="showPassword ? $t('setup.hidePassword') : $t('setup.showPassword')"
            >
              <IconEye v-if="showPassword" class="eye-icon" />
              <IconEyeOff v-else class="eye-icon" />
            </button>
          </div>
        </div>

        <!-- Connection Test Messages -->
        <div v-if="testError" class="alert error">
          <span>{{ $t('setup.testFailed', { error: testError }) }}</span>
        </div>
        <div v-if="testSuccess" class="alert success">
          <span>{{ $t('setup.testSuccess') }}</span>
        </div>
        <div v-if="saveError" class="alert error">
          <span>{{ $t('setup.saveFailed', { error: saveError }) }}</span>
        </div>

        <!-- Actions -->
        <div class="actions" style="display: flex; gap: var(--spacing-3); flex-wrap: wrap; justify-content: flex-end; width: 100%;">
          <button
            type="button"
            @click="handleTest"
            class="btn btn-secondary"
            :disabled="isTesting || isSaving || !serverUrl || !username || !password"
          >
            <span v-if="isTesting" class="spinner"></span>
            {{ isTesting ? $t('setup.testing') : $t('setup.testConnection') }}
          </button>

          <template v-if="route.query.id">
            <button
              type="button"
              @click="handleSave"
              class="btn btn-success"
              :disabled="isTesting || isSaving || !testSuccess"
            >
              <span v-if="isSaving" class="spinner"></span>
              {{ $t('setup.saveOnly') }}
            </button>

            <button
              type="button"
              @click="handleResync"
              class="btn btn-primary"
              :disabled="isTesting || isSaving || !testSuccess"
            >
              <span v-if="isSaving" class="spinner"></span>
              {{ $t('setup.resync') }}
            </button>
          </template>

          <template v-else>
            <button
              type="button"
              @click="handleSaveAndSync"
              class="btn btn-primary"
              :disabled="isTesting || isSaving || !testSuccess"
            >
              <span v-if="isSaving" class="spinner"></span>
              {{ $t('setup.saveAndSync') }}
            </button>
          </template>
        </div>
      </form>

      <!-- Sequential Syncing Progress Screen -->
      <div v-else class="sync-screen">
        <h2 class="sync-title">{{ $t('setup.syncScreen.title') }}</h2>
        <p class="sync-desc">{{ $t('setup.syncScreen.desc') }}</p>

        <div class="sync-steps">
          <div class="sync-step" :class="{ active: liveStatus.is_syncing, done: liveStatus.fetched_at }">
            <div class="step-indicator">
              <span v-if="liveStatus.is_syncing" class="spinner small"></span>
              <span v-else-if="liveStatus.fetched_at">✓</span>
              <span v-else>•</span>
            </div>
            <div class="step-details">
              <h3>{{ $t('setup.syncScreen.live') }}</h3>
              <span v-if="liveStatus.is_syncing" class="status-badge">{{ liveStatus.status || $t('setup.syncScreen.syncing') }}</span>
              <span v-else-if="liveStatus.fetched_at" class="status-badge success">{{ $t('setup.syncScreen.syncedCount', { count: liveStatus.item_count }) }}</span>
              <span v-else class="status-badge pending">{{ $t('setup.syncScreen.pending') }}</span>
            </div>
          </div>

          <div class="sync-step" :class="{ active: vodStatus.is_syncing, done: vodStatus.fetched_at }">
            <div class="step-indicator">
              <span v-if="vodStatus.is_syncing" class="spinner small"></span>
              <span v-else-if="vodStatus.fetched_at">✓</span>
              <span v-else>•</span>
            </div>
            <div class="step-details">
              <h3>{{ $t('setup.syncScreen.vod') }}</h3>
              <span v-if="vodStatus.is_syncing" class="status-badge">{{ vodStatus.status || $t('setup.syncScreen.syncing') }}</span>
              <span v-else-if="vodStatus.fetched_at" class="status-badge success">{{ $t('setup.syncScreen.syncedCount', { count: vodStatus.item_count }) }}</span>
              <span v-else class="status-badge pending">{{ $t('setup.syncScreen.pending') }}</span>
            </div>
          </div>

          <div class="sync-step" :class="{ active: seriesStatus.is_syncing, done: seriesStatus.fetched_at }">
            <div class="step-indicator">
              <span v-if="seriesStatus.is_syncing" class="spinner small"></span>
              <span v-else-if="seriesStatus.fetched_at">✓</span>
              <span v-else>•</span>
            </div>
            <div class="step-details">
              <h3>{{ $t('setup.syncScreen.series') }}</h3>
              <span v-if="seriesStatus.is_syncing" class="status-badge">{{ seriesStatus.status || $t('setup.syncScreen.syncing') }}</span>
              <span v-else-if="seriesStatus.fetched_at" class="status-badge success">{{ $t('setup.syncScreen.syncedCount', { count: seriesStatus.item_count }) }}</span>
              <span v-else class="status-badge pending">{{ $t('setup.syncScreen.pending') }}</span>
            </div>
          </div>
        </div>

        <div v-if="syncError" class="alert error sync-err-alert">
          <span>{{ $t('setup.syncScreen.failed', { error: syncError }) }}</span>
          <button @click="isSyncing = false" class="btn btn-secondary btn-retry">{{ $t('setup.syncScreen.back') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.setup-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: radial-gradient(circle at top left, #1e293b, #0f172a 70%);
  padding: var(--spacing-6);
  
  /* Force dark mode variables so label/text contrasts are perfect */
  --color-bg: #0f172a;
  --color-surface: #1e293b;
  --color-text: #f8fafc;
  --color-text-muted: #94a3b8;
  --color-primary: #60a5fa;
  --color-primary-hover: #3b82f6;
  --color-border: #334155;
  color: var(--color-text);
}

.glass-card {
  width: 100%;
  max-width: 500px;
  background: rgba(30, 41, 59, 0.7);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--radius-lg);
  padding: var(--spacing-8);
  box-shadow: var(--shadow-md), 0 0 40px rgba(96, 165, 250, 0.05);
}

.header {
  text-align: center;
  margin-bottom: var(--spacing-6);
}

.setup-logo-svg {
  width: 48px;
  height: 48px;
  color: var(--color-primary);
  margin-bottom: var(--spacing-3);
  filter: drop-shadow(0 0 12px rgba(96, 165, 250, 0.4));
  display: inline-block;
}

.glow-title {
  font-size: 2.2rem;
  color: #fff;
  text-shadow: 0 0 10px rgba(96, 165, 250, 0.4);
  font-weight: 800;
  letter-spacing: -0.025em;
}

.subtitle {
  color: var(--color-text-muted);
  margin-top: var(--spacing-2);
  font-size: 0.95rem;
}

.setup-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
}

.password-wrapper {
  position: relative;
  display: flex;
}

.password-wrapper input {
  width: 100%;
  padding-right: 42px !important;
}

.toggle-password {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  padding: 0;
  color: var(--color-text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color var(--transition-fast);
}

.toggle-password:hover {
  color: var(--color-text);
}

.eye-icon {
  width: 20px;
  height: 20px;
}

.form-group label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
}

.form-group input {
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-3) var(--spacing-4);
  color: #fff;
  font-family: inherit;
  font-size: 0.95rem;
  transition: all var(--transition-fast);
}

.form-group input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.15);
}

.form-group input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.alert {
  padding: var(--spacing-3) var(--spacing-4);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
  line-height: 1.4;
}

.alert.error {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #f87171;
}

.alert.success {
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.2);
  color: #4ade80;
}

.actions {
  display: flex;
  gap: var(--spacing-3);
  margin-top: var(--spacing-2);
}

.btn {
  flex: 1;
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
}

.btn-primary {
  background: var(--color-primary);
  color: #ffffff;
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-primary-hover);
  transform: translateY(-1px);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--color-border);
  color: var(--color-text);
}

.btn-secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none !important;
}

.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.spinner.small {
  width: 14px;
  height: 14px;
  border-width: 1.5px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Sync screen styling */
.sync-screen {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.sync-title {
  font-size: 1.5rem;
  color: #fff;
  font-weight: 700;
}

.sync-desc {
  color: var(--color-text-muted);
  font-size: 0.9rem;
  margin-top: var(--spacing-2);
  margin-bottom: var(--spacing-6);
}

.sync-steps {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
  margin-bottom: var(--spacing-6);
}

.sync-step {
  display: flex;
  align-items: center;
  gap: var(--spacing-4);
  background: rgba(15, 23, 42, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.04);
  padding: var(--spacing-4);
  border-radius: var(--radius-md);
  transition: all var(--transition-normal);
}

.sync-step.active {
  border-color: rgba(96, 165, 250, 0.3);
  background: rgba(96, 165, 250, 0.04);
}

.sync-step.done {
  border-color: rgba(34, 197, 94, 0.3);
  background: rgba(34, 197, 94, 0.04);
}

.step-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 50%;
  color: var(--color-text-muted);
  font-weight: bold;
}

.active .step-indicator {
  background: rgba(96, 165, 250, 0.15);
  color: var(--color-primary);
}

.done .step-indicator {
  background: rgba(34, 197, 94, 0.15);
  color: #4ade80;
}

.step-details {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
}

.step-details h3 {
  font-size: 1rem;
  color: var(--color-text-muted);
  transition: color var(--transition-fast);
}

.active h3, .done h3 {
  color: #fff;
}

.status-badge {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: capitalize;
  color: var(--color-text-muted);
}

.status-badge.success {
  color: #4ade80;
}

.status-badge.pending {
  opacity: 0.6;
}

.sync-err-alert {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-2);
}

.btn-retry {
  padding: var(--spacing-2) var(--spacing-3);
  font-size: 0.85rem;
  cursor: pointer;
}
</style>
