export interface VodCategory {
  profile_id: number
  category_id: string
  category_name: string
}

export interface VodStream {
  profile_id: number
  stream_id: number
  name: string | null
  stream_icon: string | null
  category_id: string | null
  rating: string | null
  container_extension: string | null
  added: string | null
  is_favorite?: number
}

export interface VodInfo {
  profile_id: number
  stream_id: number
  info_json: string
  fetched_at: string
}
