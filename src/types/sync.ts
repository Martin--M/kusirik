export type DataType = 'live_streams' | 'vod_streams' | 'series' | 'epg'

export interface SyncStatus {
  data_type: DataType
  fetched_at: string | null
  item_count: number | null
  last_error: string | null
  is_syncing: boolean
  percent: number | null
}

export interface SyncStartedEvent {
  data_type: DataType
}

export interface SyncProgressEvent {
  data_type: DataType
  percent: number
}

export interface SyncDoneEvent {
  data_type: DataType
  count: number
}

export interface SyncErrorEvent {
  data_type: DataType
  message: string
}
