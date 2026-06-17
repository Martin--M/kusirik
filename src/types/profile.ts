export interface Profile {
  id: number
  name: string
  server_url: string
  username: string
  epg_mode: 'xmltv' | 'short_epg'
  created_at: string
}

export interface SaveProfilePayload {
  name: string
  server_url: string
  username: string
  password: string
}
