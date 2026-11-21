export type MediaSortField = 'name' | 'rating' | 'added' | 'releaseDate'
export type MediaSortOrder = 'asc' | 'desc'

export interface SortableMediaItem {
  name: string | null
  rating: string | null
  release_date?: number | null
}

/**
 * Returns a new sorted array of media items (movies or TV series) according to
 * the selected sort field and direction.
 *
 * Direction semantics match FilterHeader conventions:
 * - `name`: `asc` = A->Z, `desc` = Z->A
 * - `rating`: `asc` = highest->lowest, `desc` = lowest->highest
 * - `added`: `asc` = most recent->oldest, `desc` = oldest->most recent
 * - `releaseDate`: `asc` = newest->oldest, `desc` = oldest->newest (missing dates last),
 *   with `getAddedTimestamp` as secondary tie-breaker and `name` as final tie-breaker.
 */
export function sortMediaItems<T extends SortableMediaItem>(
  items: readonly T[],
  sortField: MediaSortField,
  sortOrder: MediaSortOrder,
  getAddedTimestamp: (item: T) => number,
): T[] {
  return [...items].sort((a, b) => {
    if (sortField === 'name') {
      const nameA = a.name || ''
      const nameB = b.name || ''
      return sortOrder === 'asc'
        ? nameA.localeCompare(nameB)
        : nameB.localeCompare(nameA)
    }

    if (sortField === 'rating') {
      const ratingA = parseFloat(a.rating || '0') || 0
      const ratingB = parseFloat(b.rating || '0') || 0
      return sortOrder === 'asc' ? ratingB - ratingA : ratingA - ratingB
    }

    if (sortField === 'added') {
      const addedA = getAddedTimestamp(a)
      const addedB = getAddedTimestamp(b)
      return sortOrder === 'asc' ? addedB - addedA : addedA - addedB
    }

    if (sortField === 'releaseDate') {
      const tsA = a.release_date || 0
      const tsB = b.release_date || 0
      if (tsA === 0 && tsB !== 0) return 1
      if (tsB === 0 && tsA !== 0) return -1
      if (tsA !== tsB) {
        return sortOrder === 'asc' ? tsB - tsA : tsA - tsB
      }
      const addedA = getAddedTimestamp(a)
      const addedB = getAddedTimestamp(b)
      if (addedA !== addedB) {
        return sortOrder === 'asc' ? addedB - addedA : addedA - addedB
      }
      return (a.name || '').localeCompare(b.name || '')
    }

    return 0
  })
}
