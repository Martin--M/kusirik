import { useQuery } from '@tanstack/vue-query'
import { getLiveCategories, getLiveStreams } from '@/lib/tauri-commands'
import { PROFILE_ID } from '@/stores/profile.store'
import type { MaybeRefOrGetter } from 'vue'
import { toValue } from 'vue'

export function useLiveCategories() {
  return useQuery({
    queryKey: ['live_streams', 'categories'],
    queryFn: () => getLiveCategories(PROFILE_ID),
  })
}

export function useLiveStreams(categoryId?: MaybeRefOrGetter<string | undefined>) {
  return useQuery({
    queryKey: ['live_streams', 'streams', categoryId],
    queryFn: () => getLiveStreams(PROFILE_ID, toValue(categoryId), 0, 10000),
  })
}
