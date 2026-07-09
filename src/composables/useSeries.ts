import { useQuery } from '@tanstack/vue-query'
import { getSeriesCategories, getSeriesList, getSeriesInfo } from '@/lib/tauri-commands'
import type { MaybeRefOrGetter } from 'vue'
import { toValue } from 'vue'

export function useSeriesCategories(profileId?: MaybeRefOrGetter<number | null | undefined>) {
  return useQuery({
    queryKey: ['series', 'categories', () => (profileId !== undefined ? toValue(profileId) : undefined)],
    queryFn: async () => {
      const pid = profileId !== undefined ? toValue(profileId) : undefined
      const res = await getSeriesCategories(pid)
      res.forEach(Object.freeze)
      return Object.freeze(res)
    },
  })
}

export function useSeries(
  categoryId?: MaybeRefOrGetter<string | undefined>,
  profileId?: MaybeRefOrGetter<number | null | undefined>
) {
  return useQuery({
    queryKey: [
      'series',
      'list',
      () => (profileId !== undefined ? toValue(profileId) : undefined),
      () => toValue(categoryId)
    ],
    queryFn: async () => {
      const pid = profileId !== undefined ? toValue(profileId) : undefined
      const res = await getSeriesList(pid, toValue(categoryId), 0, 10000)
      res.forEach(Object.freeze)
      return Object.freeze(res)
    },
  })
}

export function useSeriesInfo(seriesId: MaybeRefOrGetter<number>, profileId?: MaybeRefOrGetter<number | null | undefined>) {
  return useQuery({
    queryKey: [
      'series_info',
      () => (profileId !== undefined ? toValue(profileId) : undefined),
      () => toValue(seriesId)
    ],
    queryFn: () => {
      const pid = profileId !== undefined ? toValue(profileId) : undefined
      return getSeriesInfo(pid, toValue(seriesId))
    },
    enabled: () => toValue(seriesId) > 0,
  })
}
