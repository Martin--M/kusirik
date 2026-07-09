import { useQuery } from '@tanstack/vue-query'
import { getVodCategories, getVodStreams, getVodInfo } from '@/lib/tauri-commands'
import type { MaybeRefOrGetter } from 'vue'
import { toValue } from 'vue'

export function useVodCategories(profileId?: MaybeRefOrGetter<number | null | undefined>) {
  return useQuery({
    queryKey: ['vod_streams', 'categories', () => (profileId !== undefined ? toValue(profileId) : undefined)],
    queryFn: async () => {
      const pid = profileId !== undefined ? toValue(profileId) : undefined
      const res = await getVodCategories(pid)
      res.forEach(Object.freeze)
      return Object.freeze(res)
    },
  })
}

export function useVodStreams(
  categoryId?: MaybeRefOrGetter<string | undefined>,
  profileId?: MaybeRefOrGetter<number | null | undefined>
) {
  return useQuery({
    queryKey: [
      'vod_streams',
      'streams',
      () => (profileId !== undefined ? toValue(profileId) : undefined),
      () => toValue(categoryId)
    ],
    queryFn: async () => {
      const pid = profileId !== undefined ? toValue(profileId) : undefined
      const res = await getVodStreams(pid, toValue(categoryId), 0, 10000)
      res.forEach(Object.freeze)
      return Object.freeze(res)
    },
  })
}

export function useVodInfo(streamId: MaybeRefOrGetter<number>, profileId?: MaybeRefOrGetter<number | null | undefined>) {
  return useQuery({
    queryKey: [
      'vod_info',
      () => (profileId !== undefined ? toValue(profileId) : undefined),
      () => toValue(streamId)
    ],
    queryFn: () => {
      const pid = profileId !== undefined ? toValue(profileId) : undefined
      return getVodInfo(pid, toValue(streamId))
    },
    enabled: () => toValue(streamId) > 0,
  })
}
