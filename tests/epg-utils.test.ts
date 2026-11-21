import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import type { EpgEntry } from '../src/types/epg.ts'
import { deduplicateEpgEntries, mergeShortSegments } from '../src/lib/epg-utils.ts'

const BASE = 1795269600 // 2026-11-21T14:00:00Z

function makeEntry(
  id: number,
  offsetMinStart: number,
  offsetMinStop: number,
  title: string,
): EpgEntry {
  return {
    id,
    profile_id: 1,
    channel_id: 'ch1',
    start: BASE + offsetMinStart * 60,
    stop: BASE + offsetMinStop * 60,
    title,
    description: null,
  }
}

describe('epg-utils', () => {
  it('handles empty and single-entry lists without mutating input', () => {
    assert.deepEqual(deduplicateEpgEntries([]), [])
    const single = [makeEntry(1, 0, 60, 'News')]
    assert.equal(deduplicateEpgEntries(single).length, 1)
  })

  it('sorts out-of-order entries, drops duplicates within 5 min, and keeps distinct airings', () => {
    const entries = [
      makeEntry(3, 60, 120, 'News'), // same title 60 min later -> distinct airing, MUST be kept
      makeEntry(1, 0, 60, 'News'),
      makeEntry(2, 0, 60, 'News Duplicate Exact'), // exact same start -> dropped
      makeEntry(4, 3, 60, 'News'), // same title within 5 min -> dropped
    ]

    const result = deduplicateEpgEntries(entries)
    assert.equal(result.length, 2)
    assert.equal(result[0].start, BASE)
    assert.equal(result[0].stop, BASE + 60 * 60)
    assert.equal(result[1].start, BASE + 60 * 60)
    assert.equal(result[1].stop, BASE + 120 * 60)
  })

  it('trims overlapping end times when next program starts early', () => {
    const entries = [
      makeEntry(1, 0, 75, 'Movie'),
      makeEntry(2, 60, 120, 'Late Show'),
    ]

    const result = deduplicateEpgEntries(entries)
    assert.equal(result.length, 2)
    assert.equal(result[0].stop, BASE + 60 * 60)
    assert.equal(result[1].start, BASE + 60 * 60)
  })

  it('merges short segments (<= 5 min) into neighboring entries without collapsing normal shows', () => {
    const entries = [
      makeEntry(1, 0, 60, 'Show A'),
      makeEntry(2, 60, 64, 'Commercial Bumper'), // 4 min -> merged into Show A
      makeEntry(3, 64, 124, 'Show B'),
    ]

    const result = mergeShortSegments(entries)
    assert.equal(result.length, 2)
    assert.equal(result[0].title, 'Show A')
    assert.equal(result[0].stop, BASE + 64 * 60)
    assert.equal(result[1].title, 'Show B')
    assert.equal(result[1].start, BASE + 64 * 60)
  })

  it('merges a leading short segment into the following program', () => {
    const entries = [
      makeEntry(1, 0, 3, 'Intro'),
      makeEntry(2, 3, 60, 'Main Program'),
    ]

    const result = mergeShortSegments(entries)
    assert.equal(result.length, 1)
    assert.equal(result[0].title, 'Main Program')
    assert.equal(result[0].start, BASE)
    assert.equal(result[0].stop, BASE + 60 * 60)
  })
})
