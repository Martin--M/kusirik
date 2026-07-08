import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { getPlaybackHistory, removeFromPlaybackHistory, clearPlaybackHistory } from '@/lib/tauri-commands'
import { PROFILE_ID } from '@/stores/profile.store'

export function useHistory() {
  return useQuery({
    queryKey: ['playback_history', PROFILE_ID],
    queryFn: () => getPlaybackHistory(PROFILE_ID),
  })
}

export function useRemoveFromHistory() {
  const queryClient = useQueryClient()

  const remove = async (mediaType: 'live' | 'vod' | 'series', streamId: number) => {
    await removeFromPlaybackHistory(PROFILE_ID, mediaType, streamId)
    queryClient.invalidateQueries({ queryKey: ['playback_history'] })
  }

  return { remove }
}

export function useClearHistory() {
  const queryClient = useQueryClient()

  const clear = async () => {
    await clearPlaybackHistory(PROFILE_ID)
    queryClient.invalidateQueries({ queryKey: ['playback_history'] })
  }

  return { clear }
}
