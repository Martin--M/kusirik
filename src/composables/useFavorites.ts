import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { getFavorites, toggleFavorite } from '@/lib/tauri-commands'
import { PROFILE_ID } from '@/stores/profile.store'

export function useFavorites() {
  return useQuery({
    queryKey: ['favorites', PROFILE_ID],
    queryFn: () => getFavorites(PROFILE_ID),
  })
}

export function useToggleFavorite() {
  const queryClient = useQueryClient()

  const toggle = async (mediaType: 'live' | 'vod' | 'series', streamId: number) => {
    const isFav = await toggleFavorite(PROFILE_ID, mediaType, streamId)
    
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
