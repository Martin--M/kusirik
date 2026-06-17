import { format, formatDistanceToNow, parseISO, isValid } from 'date-fns'

/**
 * Format a UTC ISO 8601 EPG timestamp to a human-readable time.
 * e.g. "2026-06-17T14:00:00Z" → "14:00"
 */
export function formatEpgTime(isoUtc: string): string {
  const d = parseISO(isoUtc)
  if (!isValid(d)) return '—'
  return format(d, 'HH:mm')
}

/**
 * Format a UTC ISO 8601 timestamp to a date + time string.
 * e.g. "2026-06-17T14:00:00Z" → "Jun 17, 14:00"
 */
export function formatDateTime(isoUtc: string): string {
  const d = parseISO(isoUtc)
  if (!isValid(d)) return '—'
  return format(d, 'MMM d, HH:mm')
}

/**
 * Relative time from now.
 * e.g. "2026-06-16T14:00:00Z" → "about 1 day ago"
 */
export function formatRelative(isoUtc: string): string {
  const d = parseISO(isoUtc)
  if (!isValid(d)) return '—'
  return formatDistanceToNow(d, { addSuffix: true })
}

/**
 * Format duration in seconds to "HH:MM" or "MM:SS" style.
 */
export function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) {
    return `${h}h ${m}m`
  }
  return `${m}:${String(s).padStart(2, '0')}`
}

/**
 * Clamp a rating string to a float for display.
 * e.g. "7.9" → "7.9" | "N/A" → null
 */
export function formatRating(raw: string | null | undefined): string | null {
  if (!raw) return null
  const n = parseFloat(raw)
  if (isNaN(n)) return null
  return n.toFixed(1)
}
