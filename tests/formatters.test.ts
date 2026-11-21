import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import {
  formatEpgTime,
  formatDateTime,
  formatRelative,
  formatDuration,
  formatRating,
} from '../src/lib/formatters.ts'

describe('formatters', () => {
  it('formats valid ISO strings and falls back to em dash for invalid inputs', () => {
    assert.match(formatEpgTime('2026-11-21T14:30:00Z'), /^\d{2}:\d{2}$/)
    assert.equal(formatEpgTime('invalid'), '—')

    assert.match(formatDateTime('2026-11-21T14:30:00Z'), /^[A-Z][a-z]{2} \d{1,2}, \d{2}:\d{2}$/)
    assert.equal(formatDateTime('invalid'), '—')

    const recentIso = new Date(Date.now() - 60_000).toISOString()
    assert.match(formatRelative(recentIso), /ago/)
    assert.equal(formatRelative('not-a-date'), '—')
  })

  it('formats durations across seconds, minutes, and hours', () => {
    assert.equal(formatDuration(5), '0:05')
    assert.equal(formatDuration(125), '2:05')
    assert.equal(formatDuration(3600), '1h 0m')
    assert.equal(formatDuration(5430), '1h 30m')
  })

  it('formats numeric ratings and returns null for invalid or empty values', () => {
    assert.equal(formatRating('7.9'), '7.9')
    assert.equal(formatRating('8'), '8.0')
    assert.equal(formatRating('6.456'), '6.5')
    assert.equal(formatRating('N/A'), null)
    assert.equal(formatRating(''), null)
    assert.equal(formatRating(null), null)
    assert.equal(formatRating(undefined), null)
  })
})
