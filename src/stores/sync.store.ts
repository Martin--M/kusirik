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
  const statuses = ref<Record<DataType, SyncStatus>>(
    Object.fromEntries(
      ALL_DATA_TYPES.map((t) => [t, initialStatus(t)])
    ) as Record<DataType, SyncStatus>
  )

  function onStarted(data_type: DataType) {
    statuses.value[data_type].is_syncing = true
    statuses.value[data_type].status = 'connecting'
    statuses.value[data_type].last_error = null
  }

  function onProgress(data_type: DataType, status: string) {
    statuses.value[data_type].status = status
  }

  function onDone(data_type: DataType, count: number, fetched_at: string) {
    statuses.value[data_type].is_syncing = false
    statuses.value[data_type].status = null
    statuses.value[data_type].item_count = count
    statuses.value[data_type].fetched_at = fetched_at
    statuses.value[data_type].last_error = null
  }

  function onError(data_type: DataType, message: string) {
    statuses.value[data_type].is_syncing = false
    statuses.value[data_type].status = null
    statuses.value[data_type].last_error = message
  }

  function isManualSyncAllowed(data_type: DataType): boolean {
    const s = statuses.value[data_type]
    // Manual sync only available if last auto-sync failed (per plan §8.8)
    return s.last_error !== null && !s.is_syncing
  }

  return { statuses, onStarted, onProgress, onDone, onError, isManualSyncAllowed }
})
