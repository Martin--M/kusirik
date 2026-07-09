import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { getFavorites, toggleFavorite } from '@/lib/tauri-commands'
import { useProfileStore } from '@/stores/profile.store'

export function useFavorites() {
  const profileStore = useProfileStore()
  return useQuery({
    queryKey: ['favorites', () => profileStore.filterProfileId],
    queryFn: () => getFavorites(profileStore.filterProfileId),
  })
}

export function useToggleFavorite() {
  const queryClient = useQueryClient()
  const profileStore = useProfileStore()

  const toggle = async (mediaType: 'live' | 'vod' | 'series', streamId: number, profileId?: number) => {
    const targetProfileId = profileId ?? profileStore.profile?.id
    if (targetProfileId === undefined) {
      throw new Error('Cannot toggle favorite: No active profile is loaded.')
    }
    const isFav = await toggleFavorite(targetProfileId, mediaType, streamId)
    
    // Invalidate relevant queries so the cache updates
    queryClient.invalidateQueries({ queryKey: ['favorites'] })
    queryClient.invalidateQueries({ queryKey: ['live_streams', 'streams'] })
    queryClient.invalidateQueries({ queryKey: ['vod_streams', 'streams'] })
    queryClient.invalidateQueries({ queryKey: ['series', 'list'] })
    queryClient.invalidateQueries({ queryKey: ['search'] })
    
    return isFav
  }

  return { toggle }
}
