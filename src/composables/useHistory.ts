import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { getPlaybackHistory, removeFromPlaybackHistory, clearPlaybackHistory } from '@/lib/tauri-commands'
import { useProfileStore } from '@/stores/profile.store'

export function useHistory() {
  const profileStore = useProfileStore()
  return useQuery({
    queryKey: ['playback_history', () => profileStore.filterProfileId],
    queryFn: () => getPlaybackHistory(profileStore.filterProfileId),
  })
}

export function useRemoveFromHistory() {
  const queryClient = useQueryClient()
  const profileStore = useProfileStore()

  const remove = async (mediaType: 'live' | 'vod' | 'series', streamId: number, profileId?: number) => {
    const targetProfileId = profileId ?? profileStore.profile?.id ?? 1
    await removeFromPlaybackHistory(targetProfileId, mediaType, streamId)
    queryClient.invalidateQueries({ queryKey: ['playback_history'] })
  }

  return { remove }
}

export function useClearHistory() {
  const queryClient = useQueryClient()
  const profileStore = useProfileStore()

  const clear = async () => {
    const targetProfileId = profileStore.profile?.id ?? 1
    await clearPlaybackHistory(targetProfileId)
    queryClient.invalidateQueries({ queryKey: ['playback_history'] })
  }

  return { clear }
}
