import { useQuery } from '@tanstack/vue-query'
import { getSeriesCategories, getSeriesList, getSeriesInfo } from '@/lib/tauri-commands'
import { PROFILE_ID } from '@/stores/profile.store'
import type { MaybeRefOrGetter } from 'vue'
import { toValue } from 'vue'

export function useSeriesCategories() {
  return useQuery({
    queryKey: ['series', 'categories'],
    queryFn: () => getSeriesCategories(PROFILE_ID),
  })
}

export function useSeries(categoryId?: MaybeRefOrGetter<string | undefined>) {
  return useQuery({
    queryKey: ['series', 'list', categoryId],
    queryFn: () => getSeriesList(PROFILE_ID, toValue(categoryId), 0, 10000),
  })
}

export function useSeriesInfo(seriesId: MaybeRefOrGetter<number>) {
  return useQuery({
    queryKey: ['series_info', seriesId],
    queryFn: () => getSeriesInfo(PROFILE_ID, toValue(seriesId)),
    enabled: () => toValue(seriesId) > 0,
  })
}
