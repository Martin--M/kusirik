export type DataType = 'live_streams' | 'vod_streams' | 'series' | 'epg'

export interface SyncStatus {
  data_type: DataType
  fetched_at: string | null
  item_count: number | null
  last_error: string | null
  is_syncing: boolean
  status: string | null // "connecting", "downloading", "parsing", "writing"
}

export interface SyncStartedEvent {
  profile_id: number
  data_type: DataType
}

export interface SyncProgressEvent {
  profile_id: number
  data_type: DataType
  status: string // "connecting", "downloading", "parsing", "writing"
}

export interface SyncDoneEvent {
  profile_id: number
  data_type: DataType
  count: number
}

export interface SyncErrorEvent {
  profile_id: number
  data_type: DataType
  message: string
}
