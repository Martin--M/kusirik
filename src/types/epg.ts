export interface EpgEntry {
  id: number
  profile_id: number
  channel_id: string
  start: string // UTC ISO 8601
  stop: string  // UTC ISO 8601
  title: string | null
  description: string | null
}
