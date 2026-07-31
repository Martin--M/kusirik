/**
 * Type-safe wrappers around Tauri invoke() calls.
 * All backend command names are centralised here — never call invoke() directly from components.
 */
import { invoke as tauriInvoke } from '@tauri-apps/api/core'

function isTauri(): boolean {
  return typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__ !== undefined
}

async function invoke<T>(cmd: string, args?: Record<string, any>): Promise<T> {
  if (!isTauri()) {
    throw new Error(
      `Tauri environment not detected. Please run the application using 'npm run tauri:dev' or 'npm run tauri:android:dev' instead of a standard web browser.`
    )
  }
  return tauriInvoke<T>(cmd, args)
}
import type { Profile, SaveProfilePayload } from '@/types/profile'
import type { LiveCategory, LiveStream } from '@/types/stream'
import type { VodCategory, VodStream } from '@/types/vod'
import type { SeriesCategory, Series } from '@/types/series'
import type { EpgEntry, GuideChannel } from '@/types/epg'
import type { SyncStatus, DataType } from '@/types/sync'

// ─── Profile ─────────────────────────────────────────────────────────────────

export const saveProfile = (payload: SaveProfilePayload) =>
  invoke<Profile>('save_profile', { payload })

export const getProfiles = () =>
  invoke<Profile[]>('get_profiles')

export const getProfile = (id: number) =>
  invoke<Profile | null>('get_profile', { id })

export const deleteProfile = (id: number) =>
  invoke<void>('delete_profile', { id })

export const activateProfile = (id: number) =>
  invoke<void>('activate_profile', { id })

export const testConnection = (payload: any) =>
  invoke<any>('test_connection', payload)

export const addPublicIptvProfile = () =>
  invoke<Profile>('add_public_iptv_profile')

// ─── Sync ────────────────────────────────────────────────────────────────────

export const triggerSync = (profileId: number, dataType: DataType, force?: boolean) =>
  invoke<void>('trigger_sync', { profileId, dataType, force })

export const getSyncStatus = (profileId: number) =>
  invoke<SyncStatus[]>('get_sync_status', { profileId })

// ─── Live ────────────────────────────────────────────────────────────────────

export interface LiveStreamDto {
  stream: LiveStream
  current_title: string | null
}

export const getLiveCategories = (profileId?: number | null) =>
  invoke<LiveCategory[]>('get_live_categories', { profileId })

export const getLiveStreams = (profileId?: number | null, categoryId?: string, offset = 0, limit = 100) =>
  invoke<LiveStreamDto[]>('get_live_streams', { profileId, categoryId, offset, limit })

export const getStreamMirrors = (profileId: number, name: string) =>
  invoke<LiveStream[]>('get_stream_mirrors', { profileId, name })

// ─── VOD ─────────────────────────────────────────────────────────────────────

export const getVodCategories = (profileId?: number | null) =>
  invoke<VodCategory[]>('get_vod_categories', { profileId })

export const getVodStreams = (profileId?: number | null, categoryId?: string, offset = 0, limit = 100) =>
  invoke<VodStream[]>('get_vod_streams', { profileId, categoryId, offset, limit })

export const getVodInfo = (profileId: number | null | undefined, streamId: number) =>
  invoke<any>('get_vod_info', { profileId, streamId })

// ─── Series ──────────────────────────────────────────────────────────────────

export const getSeriesCategories = (profileId?: number | null) =>
  invoke<SeriesCategory[]>('get_series_categories', { profileId })

export const getSeriesList = (profileId?: number | null, categoryId?: string, offset = 0, limit = 100) =>
  invoke<Series[]>('get_series', { profileId, categoryId, offset, limit })

export const getSeriesInfo = (profileId: number | null | undefined, seriesId: number) =>
  invoke<any>('get_series_info', { profileId, seriesId })

// ─── EPG ─────────────────────────────────────────────────────────────────────

export const getEpgForChannel = (profileId: number | null | undefined, channelId: string, from: number | string, to: number | string) =>
  invoke<EpgEntry[]>('get_epg_for_channel', { profileId, channelId, from, to })

export const getEpgGuide = (profileId: number | null | undefined, from: number | string, to: number | string, offset = 0, limit = 50) =>
  invoke<GuideChannel[]>('get_epg_guide', { profileId, from, to, offset, limit })

// ─── Settings ────────────────────────────────────────────────────────────────

export const getSetting = (key: string) =>
  invoke<string | null>('get_setting', { key })

export const setSetting = (key: string, value: string) =>
  invoke<void>('set_setting', { key, value })

// ─── Player ──────────────────────────────────────────────────────────────────

export const launchPlayer = (url: string, profileId?: number) =>
  invoke<void>('launch_player', { url, profileId })

export const resolveStreamUrl = (url: string, profileId?: number) =>
  invoke<string>('resolve_stream_url', { url, profileId })

export const launchAndroidIntent = (url: string) =>
  invoke<void>('launch_android_intent', { url })

export const validateStreamUrl = (url: string, profileId?: number) =>
  invoke<boolean>('validate_stream_url', { url, profileId })

export const copyToSystemClipboard = async (text: string) => {
  if (typeof navigator !== 'undefined' && navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
    try {
      await navigator.clipboard.writeText(text)
      return
    } catch (e) {
      console.warn('navigator.clipboard.writeText failed, falling back to Rust command:', e)
    }
  }
  return invoke<void>('copy_to_clipboard', { text })
}

// ─── Search ──────────────────────────────────────────────────────────────────

export interface SearchResults {
  live: LiveStream[]
  vod: VodStream[]
  series: Series[]
}

export const searchAllMedia = (profileId: number | null | undefined, query: string) =>
  invoke<SearchResults>('search_all_media', { profileId, query })

// ─── Favorites ───────────────────────────────────────────────────────────────

export const toggleFavorite = (profileId: number, mediaType: 'live' | 'vod' | 'series', streamId: number) =>
  invoke<boolean>('toggle_favorite', { profileId, mediaType, streamId })

export const getFavorites = (profileId?: number | null) =>
  invoke<SearchResults>('get_favorites', { profileId })

// ─── Playback History ─────────────────────────────────────────────────────────

export const recordPlaybackHistory = (profileId: number, mediaType: 'live' | 'vod' | 'series', streamId: number, lastEpisodeId?: number | null) =>
  invoke<void>('record_playback_history', { profileId, mediaType, streamId, lastEpisodeId })

export const getPlaybackHistory = (profileId?: number | null) =>
  invoke<SearchResults>('get_playback_history', { profileId })

export const removeFromPlaybackHistory = (profileId: number, mediaType: 'live' | 'vod' | 'series', streamId: number) =>
  invoke<void>('remove_from_playback_history', { profileId, mediaType, streamId })

export const clearPlaybackHistory = (profileId: number) =>
  invoke<void>('clear_playback_history', { profileId })

