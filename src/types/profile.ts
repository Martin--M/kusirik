export interface Profile {
  id: number
  name: string
  server_url: string
  username: string
  password?: string
  epg_mode: 'xmltv' | 'short_epg'
  created_at: string
  profile_type: 'xtream' | 'public_iptv'
}

export interface SaveProfilePayload {
  id?: number
  name: string
  server_url: string
  username: string
  password?: string
  epg_mode?: 'xmltv' | 'short_epg'
  profile_type?: 'xtream' | 'public_iptv'
}
