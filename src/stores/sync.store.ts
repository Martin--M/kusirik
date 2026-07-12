import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { SyncStatus, DataType } from '@/types/sync'

const ALL_DATA_TYPES: DataType[] = ['live_streams', 'vod_streams', 'series', 'epg']

function initialStatus(data_type: DataType): SyncStatus {
  return {
    data_type,
    fetched_at: null,
    item_count: null,
    last_error: null,
    is_syncing: false,
    status: null,
  }
}

export const useSyncStore = defineStore('sync', () => {
  const statuses = ref<Record<number, Record<DataType, SyncStatus>>>({})

  function ensureProfile(profileId: number) {
    if (!statuses.value[profileId]) {
      statuses.value[profileId] = Object.fromEntries(
        ALL_DATA_TYPES.map((t) => [t, initialStatus(t)])
      ) as Record<DataType, SyncStatus>
    }
  }

  function onStarted(profileId: number, data_type: DataType) {
    ensureProfile(profileId)
    statuses.value[profileId][data_type].is_syncing = true
    statuses.value[profileId][data_type].status = 'connecting'
    statuses.value[profileId][data_type].last_error = null
    statuses.value[profileId][data_type].fetched_at = null
  }

  function onProgress(profileId: number, data_type: DataType, status: string) {
    ensureProfile(profileId)
    statuses.value[profileId][data_type].status = status
  }

  function onDone(profileId: number, data_type: DataType, count: number, fetched_at: string) {
    ensureProfile(profileId)
    statuses.value[profileId][data_type].is_syncing = false
    statuses.value[profileId][data_type].status = null
    statuses.value[profileId][data_type].item_count = count
    statuses.value[profileId][data_type].fetched_at = fetched_at
    statuses.value[profileId][data_type].last_error = null
  }

  function onError(profileId: number, data_type: DataType, message: string) {
    ensureProfile(profileId)
    statuses.value[profileId][data_type].is_syncing = false
    statuses.value[profileId][data_type].status = null
    statuses.value[profileId][data_type].last_error = message
  }

  function isManualSyncAllowed(profileId: number, data_type: DataType): boolean {
    ensureProfile(profileId)
    const s = statuses.value[profileId][data_type]
    return s.last_error !== null && !s.is_syncing
  }

  function reset() {
    statuses.value = {}
  }

  return { statuses, ensureProfile, onStarted, onProgress, onDone, onError, isManualSyncAllowed, reset }
})
