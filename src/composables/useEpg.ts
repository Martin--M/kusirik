import { useQuery } from '@tanstack/vue-query'
import { getEpgForChannel } from '@/lib/tauri-commands'
import { PROFILE_ID } from '@/stores/profile.store'
import type { MaybeRefOrGetter } from 'vue'
import { toValue } from 'vue'

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
  channelId: MaybeRefOrGetter<string | null>,
  hoursBack = 1,
  hoursForward = 24,
) {
  return useQuery({
    queryKey: ['epg', channelId],
    queryFn: () => {
      const id = toValue(channelId)
      if (!id) return []
      const nowMs = Date.now()
      const from = new Date(nowMs - hoursBack * 3_600_000).toISOString()
      const to   = new Date(nowMs + hoursForward * 3_600_000).toISOString()
      return getEpgForChannel(PROFILE_ID, id, from, to)
    },
    enabled: () => !!toValue(channelId),
  })
}
