import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import { sortMediaItems } from '../src/lib/media-sort.ts'

interface TestMovie {
  id: number
  name: string | null
  rating: string | null
  release_date: number | null
  added: number | null
}

const JAN_2026 = 1767225600
const JAN_2025 = 1735689600

describe('media-sort', () => {
  const sample: TestMovie[] = [
    { id: 1, name: 'Beta (2026)', rating: '6.5', release_date: JAN_2026, added: 1794000000 },
    { id: 2, name: 'Alpha (2026)', rating: '8.2', release_date: JAN_2026, added: 1795000000 },
    { id: 3, name: 'Gamma (2025)', rating: '9.1', release_date: JAN_2025, added: 1795500000 },
    { id: 4, name: 'Undated Classic', rating: 'N/A', release_date: null, added: 1796000000 },
    { id: 5, name: 'Delta (2026)', rating: null, release_date: JAN_2026, added: 1794000000 },
  ]

  it('sorts by releaseDate newest first with added and alphabetical tie-breakers, missing dates last', () => {
    const sorted = sortMediaItems(sample, 'releaseDate', 'asc', (m) => m.added || 0)
    // 2026 newest-added first: Alpha (1795000000), then Beta & Delta (same added -> alphabetical),
    // then 2025 Gamma, then Undated Classic last.
    assert.deepEqual(
      sorted.map((m) => m.id),
      [2, 1, 5, 3, 4],
    )
  })

  it('sorts by releaseDate oldest first with ascending added tie-breaker, missing dates still last', () => {
    const sorted = sortMediaItems(sample, 'releaseDate', 'desc', (m) => m.added || 0)
    // 2025 Gamma first, then 2026 oldest-added (Beta, Delta), then Alpha, then Undated Classic last.
    assert.deepEqual(
      sorted.map((m) => m.id),
      [3, 1, 5, 2, 4],
    )
  })

  it('sorts by added timestamp in both directions', () => {
    const newestFirst = sortMediaItems(sample, 'added', 'asc', (m) => m.added || 0)
    assert.equal(newestFirst[0].id, 4)
    assert.equal(newestFirst[1].id, 3)

    const oldestFirst = sortMediaItems(sample, 'added', 'desc', (m) => m.added || 0)
    assert.equal(oldestFirst[oldestFirst.length - 1].id, 4)
  })

  it('sorts by numeric rating treating invalid or missing ratings as 0', () => {
    const highestFirst = sortMediaItems(sample, 'rating', 'asc', (m) => m.added || 0)
    assert.deepEqual(
      highestFirst.slice(0, 3).map((m) => m.id),
      [3, 2, 1],
    )
  })

  it('sorts alphabetically by name asc and desc without mutating original array', () => {
    const originalIds = sample.map((m) => m.id)
    const asc = sortMediaItems(sample, 'name', 'asc', (m) => m.added || 0)
    const desc = sortMediaItems(sample, 'name', 'desc', (m) => m.added || 0)

    assert.deepEqual(
      asc.map((m) => m.name),
      ['Alpha (2026)', 'Beta (2026)', 'Delta (2026)', 'Gamma (2025)', 'Undated Classic'],
    )
    assert.deepEqual(
      desc.map((m) => m.name),
      ['Undated Classic', 'Gamma (2025)', 'Delta (2026)', 'Beta (2026)', 'Alpha (2026)'],
    )
    assert.deepEqual(sample.map((m) => m.id), originalIds)
  })
})
