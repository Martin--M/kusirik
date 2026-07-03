/**
 * Formats a UTC ISO-8601 date string to the channel's local broadcast timezone format
 * expected by IPTV catch-up servers (e.g. YYYY-MM-DD:HH-MM).
 */
export function formatUtcForCatchup(dateStr: string, tzOffset?: string | null): string {
  try {
    let date = new Date(dateStr)
    if (tzOffset) {
      const sign = tzOffset.startsWith('-') ? -1 : 1
      const cleaned = tzOffset.replace(/[+-]/g, '')
      const parts = cleaned.split(':')
      if (parts.length === 2) {
        const hours = parseInt(parts[0], 10)
        const minutes = parseInt(parts[1], 10)
        const totalOffsetMinutes = sign * (hours * 60 + minutes)
        date = new Date(date.getTime() + totalOffsetMinutes * 60 * 1000)
      }
    }
    const y = date.getUTCFullYear()
    const m = String(date.getUTCMonth() + 1).padStart(2, '0')
    const d = String(date.getUTCDate()).padStart(2, '0')
    const h = String(date.getUTCHours()).padStart(2, '0')
    const min = String(date.getUTCMinutes()).padStart(2, '0')
    return `${y}-${m}-${d}:${h}-${min}`
  } catch (e) {
    return ''
  }
}

/**
 * Returns the duration between start and stop times in minutes.
 */
export function getDurationMinutes(startStr: string, stopStr: string): number {
  try {
    const start = new Date(startStr).getTime()
    const stop = new Date(stopStr).getTime()
    return Math.round((stop - start) / 60000)
  } catch (e) {
    return 0
  }
}

/**
 * Returns true if the program is currently airing based on the given current time.
 */
export function isCurrentProgram(startStr: string, stopStr: string, now: Date): boolean {
  try {
    const start = new Date(startStr)
    const stop = new Date(stopStr)
    return start <= now && stop >= now
  } catch (e) {
    return false
  }
}

/**
 * Returns true if the program stop time is in the past compared to the given current time.
 */
export function isPastProgram(stopStr: string, now: Date): boolean {
  try {
    const stop = new Date(stopStr)
    return stop < now
  } catch (e) {
    return false
  }
}
