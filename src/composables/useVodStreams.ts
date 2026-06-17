import { useQuery } from '@tanstack/vue-query'
import { getVodCategories, getVodStreams, getVodInfo } from '@/lib/tauri-commands'
import { PROFILE_ID } from '@/stores/profile.store'
import type { MaybeRefOrGetter } from 'vue'
import { toValue } from 'vue'

export function useVodCategories() {
  return useQuery({
    queryKey: ['vod_streams', 'categories'],
    queryFn: () => getVodCategories(PROFILE_ID),
  })
}

export function useVodStreams(categoryId?: MaybeRefOrGetter<string | undefined>) {
  return useQuery({
    queryKey: ['vod_streams', 'streams', categoryId],
    queryFn: () => getVodStreams(PROFILE_ID, toValue(categoryId), 0, 10000),
  })
}

export function useVodInfo(streamId: MaybeRefOrGetter<number>) {
  return useQuery({
    queryKey: ['vod_info', streamId],
    queryFn: () => getVodInfo(PROFILE_ID, toValue(streamId)),
    enabled: () => toValue(streamId) > 0,
  })
}
