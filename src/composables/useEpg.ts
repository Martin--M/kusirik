import { useQuery } from '@tanstack/vue-query'
import { getEpgForChannel } from '@/lib/tauri-commands'
import type { MaybeRefOrGetter } from 'vue'
import { toValue, ref } from 'vue'

// Global ticking clock to keep track of current time across all channel rows
export const globalNow = ref(Date.now())

if (typeof window !== 'undefined') {
  setInterval(() => {
    globalNow.value = Date.now()
  }, 30000) // Update every 30 seconds
}

/**
 * Fetch EPG entries for a channel around the current time.
 * `hoursBack` and `hoursForward` define the window.
 *
 * NOTE: We intentionally use Date.toISOString() (UTC, ends in "Z") instead of
 * date-fns formatISO, because EPG entries are stored as UTC RFC-3339 strings
 * and SQLite compares them lexicographically. A local-time string like
 * "2026-06-25T10:00-04:00" would sort BEFORE "2026-06-25T14:00:00Z" even
 * though they represent the same instant, causing all rows to be missed.
 */
export function useEpg(
  profileId: MaybeRefOrGetter<number | undefined>,
  channelId: MaybeRefOrGetter<string | null>,
  hoursBack: MaybeRefOrGetter<number> = 1,
  hoursForward: MaybeRefOrGetter<number> = 24,
) {
  return useQuery({
    queryKey: ['epg', profileId, channelId, hoursBack, hoursForward],
    queryFn: () => {
      const pid = toValue(profileId)
      if (pid === undefined) {
        throw new Error('Cannot query EPG: No profile ID was supplied.')
      }
      const id = toValue(channelId)
      if (!id) return []
      const nowMs = Date.now()
      const hb = toValue(hoursBack)
      const hf = toValue(hoursForward)
      const from = new Date(nowMs - hb * 3_600_000).toISOString()
      const to   = new Date(nowMs + hf * 3_600_000).toISOString()
      return getEpgForChannel(pid, id, from, to)
    },
    enabled: () => !!toValue(channelId) && toValue(profileId) !== undefined,
    refetchInterval: 15 * 60 * 1000, // Refetch EPG data from backend every 15 minutes
  })
}
