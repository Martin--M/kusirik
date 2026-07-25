/**
 * Helper to convert a number (epoch seconds) or string date into Date object
 */
function toDate(val: number | string): Date {
  if (typeof val === 'number') {
    return new Date(val * 1000)
  }
  return new Date(val)
}

/**
 * Formats a UTC date (number or ISO string) to the channel's local broadcast timezone format
 * expected by IPTV catch-up servers (e.g. YYYY-MM-DD:HH-MM).
 */
export function formatUtcForCatchup(dateInput: number | string, tzOffset?: number | string | null): string {
  try {
    let date = toDate(dateInput)
    if (typeof tzOffset === 'number') {
      date = new Date(date.getTime() + tzOffset * 1000)
    } else if (typeof tzOffset === 'string' && tzOffset) {
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
export function getDurationMinutes(startInput: number | string, stopInput: number | string): number {
  try {
    const start = toDate(startInput).getTime()
    const stop = toDate(stopInput).getTime()
    return Math.round((stop - start) / 60000)
  } catch (e) {
    return 0
  }
}

/**
 * Returns true if the program is currently airing based on the given current time.
 */
export function isCurrentProgram(startInput: number | string, stopInput: number | string, now: Date): boolean {
  try {
    const start = toDate(startInput)
    const stop = toDate(stopInput)
    return start <= now && stop >= now
  } catch (e) {
    return false
  }
}

/**
 * Returns true if the program stop time is in the past compared to the given current time.
 */
export function isPastProgram(stopInput: number | string, now: Date): boolean {
  try {
    const stop = toDate(stopInput)
    return stop < now
  } catch (e) {
    return false
  }
}
