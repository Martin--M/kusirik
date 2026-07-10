export interface EpgEntry {
  id: number
  profile_id: number
  channel_id: string
  start: string // UTC ISO 8601
  stop: string  // UTC ISO 8601
  title: string | null
  description: string | null
  tz_offset?: string | null
}

export interface GuideChannel {
  stream_id: number
  name: string | null
  stream_icon: string | null
  epg_channel_id: string | null
  tv_archive: number
  tv_archive_duration: number
  profile_id: number
  epg_entries: EpgEntry[]
}
