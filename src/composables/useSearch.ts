import { useQuery } from '@tanstack/vue-query'
import { searchAllMedia } from '@/lib/tauri-commands'
import { PROFILE_ID } from '@/stores/profile.store'
import type { Ref } from 'vue'

export function useGlobalSearch(query: Ref<string>) {
  return useQuery({
    queryKey: ['search', query],
    queryFn: async () => {
      const q = query.value.trim()
      if (q.length < 2) {
        return { live: [], vod: [], series: [] }
      }
      return searchAllMedia(PROFILE_ID, q)
    },
    enabled: () => query.value.trim().length >= 2,
  })
}
