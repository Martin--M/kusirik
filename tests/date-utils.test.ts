import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import {
  toEpochMs,
  toDate,
  formatUtcForCatchup,
  getDurationMinutes,
  isCurrentProgram,
  isPastProgram,
} from '../src/lib/date-utils.ts'

describe('date-utils', () => {
  it('converts Unix epoch seconds and ISO strings to milliseconds and Date', () => {
    const epochSec = 1795269600 // 2026-11-21T14:00:00Z
    assert.equal(toEpochMs(epochSec), epochSec * 1000)
    assert.equal(toEpochMs('2026-11-21T14:00:00Z'), epochSec * 1000)
    assert.equal(toDate(epochSec).toISOString(), '2026-11-21T14:00:00.000Z')
    assert.equal(toDate('2026-11-21T14:00:00Z').toISOString(), '2026-11-21T14:00:00.000Z')
  })

  it('formats catch-up timestamps with numeric and string timezone offsets', () => {
    const utcEpoch = 1795269600 // 2026-11-21T14:00:00Z
    assert.equal(formatUtcForCatchup(utcEpoch), '2026-11-21:14-00')
    assert.equal(formatUtcForCatchup(utcEpoch, 7200), '2026-11-21:16-00')
    assert.equal(formatUtcForCatchup(utcEpoch, -18000), '2026-11-21:09-00')
    assert.equal(formatUtcForCatchup(utcEpoch, '+02:30'), '2026-11-21:16-30')
    assert.equal(formatUtcForCatchup(utcEpoch, '-05:00'), '2026-11-21:09-00')
    // Crosses midnight boundary
    assert.equal(formatUtcForCatchup('2026-11-21T23:30:00Z', 3600), '2026-11-22:00-30')
  })

  it('calculates program duration in rounded minutes', () => {
    assert.equal(getDurationMinutes(1795269600, 1795273200), 60)
    assert.equal(getDurationMinutes('2026-11-21T14:00:00Z', '2026-11-21T14:45:00Z'), 45)
    assert.equal(getDurationMinutes(1795269600, 1795269600 + 90), 2)
  })

  it('detects current and past programs for both epoch seconds and ISO strings', () => {
    const now = new Date('2026-11-21T14:30:00Z')
    const start = 1795269600 // 14:00Z
    const stop = 1795273200 // 15:00Z

    assert.equal(isCurrentProgram(start, stop, now), true)
    assert.equal(isCurrentProgram('2026-11-21T14:00:00Z', '2026-11-21T15:00:00Z', now), true)
    assert.equal(isCurrentProgram(stop, stop + 3600, now), false)

    assert.equal(isPastProgram(start, now), true)
    assert.equal(isPastProgram(stop, now), false)
    assert.equal(isPastProgram('2026-11-21T14:29:59Z', now), true)
  })
})
