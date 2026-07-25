export interface EpgEntry {
  id: number
  profile_id: number
  channel_id: string
  start: number // Unix Epoch Seconds
  stop: number  // Unix Epoch Seconds
  title: string | null
  description: string | null
  tz_offset?: number | null // Offset in seconds
}

export interface GuideChannel {
  stream_id: number
  name: string | null
  stream_icon: string | null
  epg_channel_id: string | null
  tv_archive: number
  tv_archive_duration: number
  profile_id: number
  countries?: string | null
  epg_entries: EpgEntry[]
}
