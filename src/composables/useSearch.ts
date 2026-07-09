import { useQuery } from '@tanstack/vue-query'
import { searchAllMedia } from '@/lib/tauri-commands'
import { useProfileStore } from '@/stores/profile.store'
import type { Ref } from 'vue'

export function useGlobalSearch(query: Ref<string>) {
  const profileStore = useProfileStore()
  return useQuery({
    queryKey: ['search', () => profileStore.filterProfileId, query],
    queryFn: async () => {
      const q = query.value.trim()
      if (q.length < 2) {
        return { live: [], vod: [], series: [] }
      }
      return searchAllMedia(profileStore.filterProfileId, q)
    },
    enabled: () => query.value.trim().length >= 2,
  })
}
