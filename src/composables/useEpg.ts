import { useQuery } from '@tanstack/vue-query'
import { getEpgForChannel } from '@/lib/tauri-commands'
import { PROFILE_ID } from '@/stores/profile.store'
import type { MaybeRefOrGetter } from 'vue'
import { toValue } from 'vue'
import { subHours, addHours, formatISO } from 'date-fns'

/**
 * Fetch EPG entries for a channel around the current time.
 * `hoursBack` and `hoursForward` define the window.
 */
export function useEpg(
  channelId: MaybeRefOrGetter<string | null>,
  hoursBack = 1,
  hoursForward = 4,
) {
  return useQuery({
    queryKey: ['epg', channelId],
    queryFn: () => {
      const id = toValue(channelId)
      if (!id) return []
      const now = new Date()
      const from = formatISO(subHours(now, hoursBack))
      const to = formatISO(addHours(now, hoursForward))
      return getEpgForChannel(PROFILE_ID, id, from, to)
    },
    enabled: () => !!toValue(channelId),
  })
}
