import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useQueryClient } from '@tanstack/vue-query'
import { useSyncStore } from '@/stores/sync.store'
import type {
  SyncStartedEvent,
  SyncProgressEvent,
  SyncDoneEvent,
  SyncErrorEvent,
} from '@/types/sync'

/**
 * Listens to sync:// Tauri events and:
 * 1. Updates the sync store (for UI status/progress display)
 * 2. Invalidates TanStack Query caches when a sync completes
 *    so that live, movies, series views refresh from the new DB data.
 *
 * Mount this composable once at the app root (App.vue).
 */
export function useSync() {
  const syncStore = useSyncStore()
  const queryClient = useQueryClient()
  const unlisten: UnlistenFn[] = []

  onMounted(async () => {
    unlisten.push(
      await listen<SyncStartedEvent>('sync://started', ({ payload }) => {
        syncStore.onStarted(payload.profile_id, payload.data_type)
      })
    )

    unlisten.push(
      await listen<SyncProgressEvent>('sync://progress', ({ payload }) => {
        syncStore.onProgress(payload.profile_id, payload.data_type, payload.status)
      })
    )

    unlisten.push(
      await listen<SyncDoneEvent>('sync://done', ({ payload }) => {
        const now = new Date().toISOString()
        syncStore.onDone(payload.profile_id, payload.data_type, payload.count, now)
        // Invalidate TanStack Query cache for this data type so views re-fetch from DB
        queryClient.invalidateQueries({ queryKey: [payload.data_type] })
      })
    )

    unlisten.push(
      await listen<SyncErrorEvent>('sync://error', ({ payload }) => {
        syncStore.onError(payload.profile_id, payload.data_type, payload.message)
      })
    )
  })

  onUnmounted(() => {
    unlisten.forEach((fn) => fn())
  })
}
