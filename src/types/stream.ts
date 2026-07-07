export interface LiveCategory {
  profile_id: number
  category_id: string
  category_name: string
}

export interface LiveStream {
  profile_id: number
  stream_id: number
  name: string | null
  stream_icon: string | null
  epg_channel_id: string | null
  category_id: string | null
  tv_archive: number
  tv_archive_duration: number
  added: string | null
  is_favorite?: number
  current_title?: string | null
}

export type SortOrder = 'asc' | 'desc'

