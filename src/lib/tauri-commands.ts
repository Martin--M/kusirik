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
import type { EpgEntry } from '@/types/epg'
import type { SyncStatus, DataType } from '@/types/sync'

// ─── Profile ─────────────────────────────────────────────────────────────────

export const saveProfile = (payload: SaveProfilePayload) =>
  invoke<Profile>('save_profile', { payload })

export const getProfile = (id: number) =>
  invoke<Profile | null>('get_profile', { id })

export const deleteProfile = (id: number) =>
  invoke<void>('delete_profile', { id })

export const testConnection = (payload: any) =>
  invoke<any>('test_connection', payload)

// ─── Sync ────────────────────────────────────────────────────────────────────

export const triggerSync = (dataType: DataType) =>
  invoke<void>('trigger_sync', { dataType })

export const getSyncStatus = () =>
  invoke<SyncStatus[]>('get_sync_status')

// ─── Live ────────────────────────────────────────────────────────────────────

export const getLiveCategories = (profileId: number) =>
  invoke<LiveCategory[]>('get_live_categories', { profileId })

export const getLiveStreams = (profileId: number, categoryId?: string, offset = 0, limit = 100) =>
  invoke<LiveStream[]>('get_live_streams', { profileId, categoryId, offset, limit })

// ─── VOD ─────────────────────────────────────────────────────────────────────

export const getVodCategories = (profileId: number) =>
  invoke<VodCategory[]>('get_vod_categories', { profileId })

export const getVodStreams = (profileId: number, categoryId?: string, offset = 0, limit = 100) =>
  invoke<VodStream[]>('get_vod_streams', { profileId, categoryId, offset, limit })

export const getVodInfo = (profileId: number, streamId: number) =>
  invoke<any>('get_vod_info', { profileId, streamId })

// ─── Series ──────────────────────────────────────────────────────────────────

export const getSeriesCategories = (profileId: number) =>
  invoke<SeriesCategory[]>('get_series_categories', { profileId })

export const getSeriesList = (profileId: number, categoryId?: string, offset = 0, limit = 100) =>
  invoke<Series[]>('get_series', { profileId, categoryId, offset, limit })

export const getSeriesInfo = (profileId: number, seriesId: number) =>
  invoke<any>('get_series_info', { profileId, seriesId })

// ─── EPG ─────────────────────────────────────────────────────────────────────

export const getEpgForChannel = (profileId: number, channelId: string, from: string, to: string) =>
  invoke<EpgEntry[]>('get_epg_for_channel', { profileId, channelId, from, to })

// ─── Settings ────────────────────────────────────────────────────────────────

export const getSetting = (key: string) =>
  invoke<string | null>('get_setting', { key })

export const setSetting = (key: string, value: string) =>
  invoke<void>('set_setting', { key, value })

// ─── Player ──────────────────────────────────────────────────────────────────

export const launchPlayer = (url: string) =>
  invoke<void>('launch_player', { url })

export const resolveStreamUrl = (url: string) =>
  invoke<string>('resolve_stream_url', { url })

export const launchAndroidIntent = (url: string) =>
  invoke<void>('launch_android_intent', { url })

export const copyToSystemClipboard = (text: string) =>
  invoke<void>('copy_to_clipboard', { text })

// ─── Search ──────────────────────────────────────────────────────────────────

export interface SearchResults {
  live: LiveStream[]
  vod: VodStream[]
  series: Series[]
}

export const searchAllMedia = (profileId: number, query: string) =>
  invoke<SearchResults>('search_all_media', { profileId, query })

