/**
 * Constructs Xtream Codes playback URLs.
 * Format selection follows plan §8.7:
 *   1. Use allowedFormats from server auth response (stored in settings)
 *   2. Prefer 'ts'; fall back to 'm3u8'; then first in list
 *   3. Respect user override from settings
 */

export type StreamFormat = 'ts' | 'm3u8' | string

/**
 * Pick the best live stream format given the server's allowed_output_formats
 * and the user's optional override.
 */
export function pickLiveFormat(
  allowedFormats: StreamFormat[],
  userOverride?: StreamFormat | null,
): StreamFormat {
  if (userOverride && allowedFormats.includes(userOverride)) {
    return userOverride
  }
  if (allowedFormats.includes('ts')) return 'ts'
  if (allowedFormats.includes('m3u8')) return 'm3u8'
  return allowedFormats[0] ?? 'ts'
}

export interface UrlParams {
  serverUrl: string
  username: string
  password: string
}

export function buildLiveUrl(
  params: UrlParams,
  streamId: number,
  format: StreamFormat,
): string {
  const { serverUrl, username, password } = params
  return `${serverUrl}/live/${username}/${password}/${streamId}.${format}`
}

export function buildMovieUrl(
  params: UrlParams,
  streamId: number,
  containerExtension: string,
): string {
  const { serverUrl, username, password } = params
  return `${serverUrl}/movie/${username}/${password}/${streamId}.${containerExtension}`
}

export function buildEpisodeUrl(
  params: UrlParams,
  streamId: number,
  containerExtension: string,
): string {
  const { serverUrl, username, password } = params
  return `${serverUrl}/series/${username}/${password}/${streamId}.${containerExtension}`
}

export function buildApiUrl(params: UrlParams, action?: string, extra?: Record<string, string | number>): string {
  const { serverUrl, username, password } = params
  const base = `${serverUrl}/player_api.php?username=${username}&password=${password}`
  if (!action) return base
  let url = `${base}&action=${action}`
  if (extra) {
    for (const [k, v] of Object.entries(extra)) {
      url += `&${k}=${v}`
    }
  }
  return url
}
