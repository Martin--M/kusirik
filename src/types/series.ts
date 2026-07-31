export interface SeriesCategory {
  profile_id: number
  category_id: string
  category_name: string
}

export interface Series {
  profile_id: number
  series_id: number
  name: string | null
  cover: string | null
  category_id: string | null
  rating: string | null
  plot: string | null
  cast_: string | null
  director: string | null
  genre: string | null
  release_date: number | null
  last_modified: number | null
  is_favorite?: number
  last_episode_id?: number | null
}

export interface SeriesInfo {
  profile_id: number
  series_id: number
  info_json: string
  fetched_at: string
}
