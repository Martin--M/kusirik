import { useQuery } from '@tanstack/vue-query'
import { getLiveCategories, getLiveStreams } from '@/lib/tauri-commands'
import { PROFILE_ID } from '@/stores/profile.store'
import type { MaybeRefOrGetter } from 'vue'
import { toValue } from 'vue'

export function useLiveCategories() {
  return useQuery({
    queryKey: ['live_streams', 'categories'],
    queryFn: async () => {
      const res = await getLiveCategories(PROFILE_ID)
      res.forEach(Object.freeze)
      return Object.freeze(res)
    },
  })
}

export function useLiveStreams(categoryId?: MaybeRefOrGetter<string | undefined>) {
  return useQuery({
    queryKey: ['live_streams', 'streams', categoryId],
    queryFn: async () => {
      const res = await getLiveStreams(PROFILE_ID, toValue(categoryId), 0, 10000)
      res.forEach(Object.freeze)
      return Object.freeze(res)
    },
  })
}
