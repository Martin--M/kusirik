import { useQuery } from '@tanstack/vue-query'
import { getLiveCategories, getLiveStreams } from '@/lib/tauri-commands'
import type { MaybeRefOrGetter } from 'vue'
import { toValue } from 'vue'

export function useLiveCategories(profileId?: MaybeRefOrGetter<number | null | undefined>) {
  return useQuery({
    queryKey: ['live_streams', 'categories', () => (profileId !== undefined ? toValue(profileId) : undefined)],
    queryFn: async () => {
      const pid = profileId !== undefined ? toValue(profileId) : undefined
      const res = await getLiveCategories(pid)
      res.forEach(Object.freeze)
      return Object.freeze(res)
    },
  })
}

export function useLiveStreams(
  categoryId?: MaybeRefOrGetter<string | undefined>,
  profileId?: MaybeRefOrGetter<number | null | undefined>
) {
  return useQuery({
    queryKey: [
      'live_streams',
      'streams',
      () => (profileId !== undefined ? toValue(profileId) : undefined),
      () => toValue(categoryId)
    ],
    queryFn: async () => {
      const pid = profileId !== undefined ? toValue(profileId) : undefined
      const res = await getLiveStreams(pid, toValue(categoryId), 0, 10000)
      res.forEach(Object.freeze)
      return Object.freeze(res)
    },
  })
}
