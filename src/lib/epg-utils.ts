import type { EpgEntry } from '../types/epg.ts'
import { toEpochMs } from './date-utils.ts'

/**
 * Deduplicates and trims overlapping EPG program entries for a channel,
 * then merges very short segments (<= 5 minutes) into adjacent programs.
 */
export function deduplicateEpgEntries(entries: EpgEntry[]): EpgEntry[] {
  if (!entries || entries.length === 0) return []
  const sorted = entries
    .map((e) => ({ ...e }))
    .sort((a, b) => toEpochMs(a.start) - toEpochMs(b.start))
  const result: EpgEntry[] = []

  for (const entry of sorted) {
    if (result.length === 0) {
      result.push(entry)
      continue
    }

    const last = result[result.length - 1]
    const lastStart = toEpochMs(last.start)
    const lastStop = toEpochMs(last.stop)
    const currentStart = toEpochMs(entry.start)

    // Check if duplicate (same start time or same title within 5 minutes)
    if (
      lastStart === currentStart ||
      (last.title === entry.title && Math.abs(lastStart - currentStart) < 300_000)
    ) {
      continue
    }

    // If current start is before last stop, trim the previous entry's stop time
    if (currentStart < lastStop) {
      if (currentStart > lastStart) {
        last.stop = entry.start
      } else {
        continue
      }
    }

    result.push(entry)
  }

  return mergeShortSegments(result)
}

/**
 * Merges EPG segments of 5 minutes or less into neighboring programs so the
 * timeline grid does not render unreadable sliver blocks.
 */
export function mergeShortSegments(entries: EpgEntry[]): EpgEntry[] {
  if (entries.length <= 1) return entries

  const list = entries.map((e) => ({ ...e }))
  let i = 0
  while (i < list.length) {
    const entry = list[i]
    const duration = (toEpochMs(entry.stop) - toEpochMs(entry.start)) / 60_000

    if (duration <= 5) {
      let merged = false
      const prev = i > 0 ? list[i - 1] : null
      const next = i < list.length - 1 ? list[i + 1] : null

      const prevDuration = prev
        ? (toEpochMs(prev.stop) - toEpochMs(prev.start)) / 60_000
        : Infinity
      const nextDuration = next
        ? (toEpochMs(next.stop) - toEpochMs(next.start)) / 60_000
        : Infinity

      if (prev && prevDuration <= 5) {
        prev.stop = entry.stop
        list.splice(i, 1)
        merged = true
      } else if (next && nextDuration <= 5) {
        next.start = entry.start
        list.splice(i, 1)
        merged = true
      } else if (prev) {
        prev.stop = entry.stop
        list.splice(i, 1)
        merged = true
      } else if (next) {
        next.start = entry.start
        list.splice(i, 1)
        merged = true
      }

      if (merged) {
        continue
      }
    }
    i++
  }
  return list
}
