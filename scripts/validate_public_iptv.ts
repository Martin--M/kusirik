#!/usr/bin/env node
/**
 * Concurrent validator for the iptv-org public IPTV stream catalog.
 *
 * Uses fast HTTP + HLS manifest/first-segment verification by default, with an
 * optional `--ffprobe` flag for deep codec/container verification.
 * Writes `valid.json`, `valid.csv`, `invalid.json`, `invalid.csv`, and `summary.json`.
 *
 * Usage:
 *   node scripts/validate_public_iptv.ts [options]
 *   npm run validate:iptv -- --country US,CA --limit 200
 */

import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { execFile } from 'node:child_process'
import { promisify } from 'node:util'
import { pathToFileURL } from 'node:url'

const execFileAsync = promisify(execFile)

export const DEFAULT_STREAMS_URL = 'https://iptv-org.github.io/api/streams.json'
export const DEFAULT_CHANNELS_URL = 'https://iptv-org.github.io/api/channels.json'
export const DEFAULT_USER_AGENT =
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) VLC/3.0.21'

export interface RawIptvStream {
  channel?: string | null
  title?: string | null
  url: string
  referrer?: string | null
  http_referrer?: string | null
  user_agent?: string | null
  quality?: string | null
}

export interface RawIptvChannel {
  id: string
  name: string
  country?: string | null
  categories?: string[] | null
}

export interface EnrichedStream {
  channel_id: string
  name: string
  url: string
  country: string
  categories: string[]
  referrer: string | null
  user_agent: string | null
  quality: string | null
}

export type ValidationReason =
  | 'ok'
  | 'geo_blocked'
  | 'unauthorized'
  | 'not_found'
  | 'server_error'
  | 'http_error'
  | 'invalid_content_type'
  | 'invalid_manifest'
  | 'empty_playlist'
  | 'variant_error'
  | 'segment_error'
  | 'empty_stream'
  | 'timeout'
  | 'network_error'
  | 'ffprobe_failed'

export interface ValidationResult {
  channel_id: string
  name: string
  url: string
  country: string
  categories: string
  valid: boolean
  reason: ValidationReason
  http_status: number | null
  latency_ms: number
  stream_kind: 'hls' | 'direct' | 'unknown'
  video_codec: string | null
  audio_codec: string | null
  resolution: string | null
  detail: string
}

export interface CliOptions {
  streamsUrl: string
  channelsUrl: string
  streamsFile: string | null
  channelsFile: string | null
  countries: string[]
  categories: string[]
  limit: number | null
  concurrency: number
  timeoutMs: number
  useFfprobe: boolean
  checkSegment: boolean
  outDir: string
  help: boolean
}

export function parseCliArgs(argv: string[]): CliOptions {
  let useFfprobe = false
  let concurrencyExplicit = false
  const opts: CliOptions = {
    streamsUrl: DEFAULT_STREAMS_URL,
    channelsUrl: DEFAULT_CHANNELS_URL,
    streamsFile: null,
    channelsFile: null,
    countries: [],
    categories: [],
    limit: null,
    concurrency: 50,
    timeoutMs: 8000,
    useFfprobe: false,
    checkSegment: true,
    outDir: 'iptv-validation-output',
    help: false,
  }

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]
    const next = () => {
      if (i + 1 >= argv.length) {
        throw new Error(`Missing value for ${arg}`)
      }
      return argv[++i]
    }

    switch (arg) {
      case '-h':
      case '--help':
        opts.help = true
        break
      case '--streams-url':
        opts.streamsUrl = next()
        break
      case '--channels-url':
        opts.channelsUrl = next()
        break
      case '--streams-file':
        opts.streamsFile = next()
        break
      case '--channels-file':
        opts.channelsFile = next()
        break
      case '--country':
        opts.countries = next()
          .split(',')
          .map((c) => c.trim().toUpperCase())
          .filter(Boolean)
        break
      case '--category':
        opts.categories = next()
          .split(',')
          .map((c) => c.trim().toLowerCase())
          .filter(Boolean)
        break
      case '--limit': {
        const n = Number.parseInt(next(), 10)
        if (!Number.isFinite(n) || n <= 0) {
          throw new Error('--limit must be a positive integer')
        }
        opts.limit = n
        break
      }
      case '--concurrency': {
        const n = Number.parseInt(next(), 10)
        if (!Number.isFinite(n) || n <= 0) {
          throw new Error('--concurrency must be a positive integer')
        }
        opts.concurrency = n
        concurrencyExplicit = true
        break
      }
      case '--timeout': {
        const n = Number.parseInt(next(), 10)
        if (!Number.isFinite(n) || n <= 0) {
          throw new Error('--timeout must be a positive integer (milliseconds)')
        }
        opts.timeoutMs = n
        break
      }
      case '--ffprobe':
        useFfprobe = true
        opts.useFfprobe = true
        break
      case '--skip-segment':
        opts.checkSegment = false
        break
      case '--out-dir':
        opts.outDir = next()
        break
      default:
        throw new Error(`Unknown argument: ${arg}`)
    }
  }

  if (useFfprobe && !concurrencyExplicit) {
    opts.concurrency = 15
  }

  return opts
}

export function classifyHttpStatus(status: number): ValidationReason | null {
  if (status >= 200 && status < 300) return null
  if (status === 403 || status === 451) return 'geo_blocked'
  if (status === 401) return 'unauthorized'
  if (status === 404 || status === 410) return 'not_found'
  if (status >= 500) return 'server_error'
  return 'http_error'
}

export function isMasterPlaylist(manifest: string): boolean {
  return manifest.includes('#EXT-X-STREAM-INF')
}

/**
 * Extracts the first media/variant URI from an HLS `#EXTM3U` playlist and
 * resolves it against `baseUrl`.
 */
export function extractFirstHlsUri(manifest: string, baseUrl: string): string | null {
  const lines = manifest.split(/\r?\n/)
  for (const rawLine of lines) {
    const line = rawLine.trim()
    if (!line || line.startsWith('#')) {
      continue
    }
    try {
      return new URL(line, baseUrl).toString()
    } catch {
      return null
    }
  }
  return null
}

export function enrichAndFilterStreams(
  rawStreams: RawIptvStream[],
  rawChannels: RawIptvChannel[],
  opts: Pick<CliOptions, 'countries' | 'categories' | 'limit'>,
): EnrichedStream[] {
  const channelMap = new Map<string, RawIptvChannel>()
  for (const ch of rawChannels) {
    channelMap.set(ch.id, ch)
  }

  const countrySet = new Set(opts.countries.map((c) => c.toUpperCase()))
  const categorySet = new Set(opts.categories.map((c) => c.toLowerCase()))

  const result: EnrichedStream[] = []
  for (const s of rawStreams) {
    if (!s.url) continue
    const chId = s.channel ?? ''
    const ch = chId ? channelMap.get(chId) : undefined
    const country = (ch?.country ?? '').toUpperCase()
    const categories = (ch?.categories ?? []).map((c) => c.toLowerCase())

    if (countrySet.size > 0 && !countrySet.has(country)) {
      continue
    }
    if (categorySet.size > 0 && !categories.some((c) => categorySet.has(c))) {
      continue
    }

    result.push({
      channel_id: chId,
      name: ch?.name || s.title || chId || s.url,
      url: s.url,
      country,
      categories,
      referrer: s.referrer ?? s.http_referrer ?? null,
      user_agent: s.user_agent ?? null,
      quality: s.quality ?? null,
    })

    if (opts.limit !== null && result.length >= opts.limit) {
      break
    }
  }

  return result
}

function escapeCsvCell(value: unknown): string {
  const str = value === null || value === undefined ? '' : String(value)
  if (str.includes(',') || str.includes('"') || str.includes('\n') || str.includes('\r')) {
    return `"${str.replace(/"/g, '""')}"`
  }
  return str
}

export const CSV_COLUMNS: (keyof ValidationResult)[] = [
  'channel_id',
  'name',
  'country',
  'categories',
  'url',
  'valid',
  'reason',
  'http_status',
  'latency_ms',
  'stream_kind',
  'video_codec',
  'audio_codec',
  'resolution',
  'detail',
]

export function toCsv(rows: ValidationResult[]): string {
  const header = CSV_COLUMNS.join(',')
  const body = rows.map((row) => CSV_COLUMNS.map((col) => escapeCsvCell(row[col])).join(','))
  return [header, ...body].join('\n') + '\n'
}

function buildHeaders(stream: EnrichedStream): Record<string, string> {
  const headers: Record<string, string> = {
    'User-Agent': stream.user_agent || DEFAULT_USER_AGENT,
    Accept: '*/*',
  }
  if (stream.referrer) {
    headers['Referer'] = stream.referrer
  }
  return headers
}

async function readUpToBytes(
  response: Response,
  maxBytes: number,
): Promise<Uint8Array> {
  if (!response.body) {
    return new Uint8Array(0)
  }
  const reader = response.body.getReader()
  const chunks: Uint8Array[] = []
  let total = 0
  try {
    while (total < maxBytes) {
      const { value, done } = await reader.read()
      if (done || !value) break
      const remaining = maxBytes - total
      if (value.byteLength > remaining) {
        chunks.push(value.subarray(0, remaining))
        total += remaining
        break
      }
      chunks.push(value)
      total += value.byteLength
    }
  } finally {
    await reader.cancel().catch(() => {})
  }

  const out = new Uint8Array(total)
  let offset = 0
  for (const c of chunks) {
    out.set(c, offset)
    offset += c.byteLength
  }
  return out
}

function isLikelyHls(url: string, contentType: string): boolean {
  const ct = contentType.toLowerCase()
  if (ct.includes('mpegurl') || ct.includes('m3u')) return true
  try {
    const pathname = new URL(url).pathname.toLowerCase()
    return pathname.endsWith('.m3u8') || pathname.endsWith('.m3u')
  } catch {
    return url.toLowerCase().includes('.m3u8')
  }
}

async function checkSegmentReachable(
  segmentUrl: string,
  headers: Record<string, string>,
  timeoutMs: number,
): Promise<{ ok: boolean; status: number | null; detail: string }> {
  try {
    const headRes = await fetch(segmentUrl, {
      method: 'HEAD',
      headers,
      redirect: 'follow',
      signal: AbortSignal.timeout(timeoutMs),
    })
    if (headRes.ok) {
      return { ok: true, status: headRes.status, detail: '' }
    }
    // Many IPTV/HLS origins reject HEAD (405/403/501) while allowing GET
    if (headRes.status !== 405 && headRes.status !== 403 && headRes.status !== 501) {
      return {
        ok: false,
        status: headRes.status,
        detail: `Segment returned HTTP ${headRes.status}`,
      }
    }
  } catch {
    // Fall through to ranged GET probe
  }

  const getRes = await fetch(segmentUrl, {
    method: 'GET',
    headers: { ...headers, Range: 'bytes=0-1023' },
    redirect: 'follow',
    signal: AbortSignal.timeout(timeoutMs),
  })
  if (!getRes.ok && getRes.status !== 206) {
    await getRes.body?.cancel().catch(() => {})
    return {
      ok: false,
      status: getRes.status,
      detail: `Segment returned HTTP ${getRes.status}`,
    }
  }
  const bytes = await readUpToBytes(getRes, 1024)
  if (bytes.byteLength === 0) {
    return { ok: false, status: getRes.status, detail: 'Segment returned 0 bytes' }
  }
  return { ok: true, status: getRes.status, detail: '' }
}

interface FfprobeOutcome {
  ok: boolean
  videoCodec: string | null
  audioCodec: string | null
  resolution: string | null
  detail: string
}

async function runFfprobe(
  stream: EnrichedStream,
  timeoutMs: number,
): Promise<FfprobeOutcome> {
  const args = [
    '-v',
    'error',
    '-print_format',
    'json',
    '-show_streams',
    '-rw_timeout',
    String(timeoutMs * 1000),
    '-user_agent',
    stream.user_agent || DEFAULT_USER_AGENT,
  ]
  if (stream.referrer) {
    args.push('-headers', `Referer: ${stream.referrer}\r\n`)
  }
  args.push(stream.url)

  try {
    const { stdout } = await execFileAsync('ffprobe', args, {
      timeout: timeoutMs + 2000,
      maxBuffer: 2 * 1024 * 1024,
    })
    const parsed = JSON.parse(stdout || '{}') as {
      streams?: Array<{
        codec_type?: string
        codec_name?: string
        width?: number
        height?: number
      }>
    }
    const streams = parsed.streams ?? []
    const video = streams.find((s) => s.codec_type === 'video')
    const audio = streams.find((s) => s.codec_type === 'audio')
    if (!video && !audio) {
      return {
        ok: false,
        videoCodec: null,
        audioCodec: null,
        resolution: null,
        detail: 'ffprobe found no playable video or audio streams',
      }
    }
    return {
      ok: true,
      videoCodec: video?.codec_name ?? null,
      audioCodec: audio?.codec_name ?? null,
      resolution: video?.width && video?.height ? `${video.width}x${video.height}` : null,
      detail: '',
    }
  } catch (err: unknown) {
    const message =
      err instanceof Error
        ? ((err as Error & { stderr?: string }).stderr?.trim() || err.message)
        : String(err)
    return {
      ok: false,
      videoCodec: null,
      audioCodec: null,
      resolution: null,
      detail: `ffprobe failed: ${message.slice(0, 200)}`,
    }
  }
}

export async function validateStream(
  stream: EnrichedStream,
  opts: Pick<CliOptions, 'timeoutMs' | 'checkSegment' | 'useFfprobe'>,
): Promise<ValidationResult> {
  const startMs = Date.now()
  const base: Omit<
    ValidationResult,
    'valid' | 'reason' | 'http_status' | 'latency_ms' | 'stream_kind' | 'detail'
  > = {
    channel_id: stream.channel_id,
    name: stream.name,
    url: stream.url,
    country: stream.country,
    categories: stream.categories.join(';'),
    video_codec: null,
    audio_codec: null,
    resolution: null,
  }

  const finish = (
    valid: boolean,
    reason: ValidationReason,
    httpStatus: number | null,
    streamKind: 'hls' | 'direct' | 'unknown',
    detail: string,
    media?: Partial<Pick<ValidationResult, 'video_codec' | 'audio_codec' | 'resolution'>>,
  ): ValidationResult => ({
    ...base,
    ...media,
    valid,
    reason,
    http_status: httpStatus,
    latency_ms: Date.now() - startMs,
    stream_kind: streamKind,
    detail,
  })

  const headers = buildHeaders(stream)

  try {
    const res = await fetch(stream.url, {
      method: 'GET',
      headers,
      redirect: 'follow',
      signal: AbortSignal.timeout(opts.timeoutMs),
    })

    const statusReason = classifyHttpStatus(res.status)
    if (statusReason) {
      await res.body?.cancel().catch(() => {})
      return finish(false, statusReason, res.status, 'unknown', `HTTP ${res.status}`)
    }

    const contentType = res.headers.get('content-type') ?? ''
    if (contentType.toLowerCase().includes('text/html')) {
      await res.body?.cancel().catch(() => {})
      return finish(
        false,
        'invalid_content_type',
        res.status,
        'unknown',
        `Unexpected content-type: ${contentType}`,
      )
    }

    const finalUrl = res.url || stream.url
    const hlsCandidate = isLikelyHls(finalUrl, contentType)

    if (hlsCandidate) {
      // Cap manifest read at 256 KB so misconfigured binary streams never OOM
      const bytes = await readUpToBytes(res, 256 * 1024)
      const text = new TextDecoder('utf-8', { fatal: false }).decode(bytes).trimStart()
      if (!text.startsWith('#EXTM3U')) {
        return finish(
          false,
          'invalid_manifest',
          res.status,
          'hls',
          'Response did not start with #EXTM3U',
        )
      }

      let mediaPlaylistText = text
      let mediaPlaylistUrl = finalUrl

      if (isMasterPlaylist(text)) {
        const variantUrl = extractFirstHlsUri(text, finalUrl)
        if (!variantUrl) {
          return finish(
            false,
            'empty_playlist',
            res.status,
            'hls',
            'Master playlist contained no variant streams',
          )
        }
        const varRes = await fetch(variantUrl, {
          method: 'GET',
          headers,
          redirect: 'follow',
          signal: AbortSignal.timeout(opts.timeoutMs),
        })
        const varReason = classifyHttpStatus(varRes.status)
        if (varReason) {
          await varRes.body?.cancel().catch(() => {})
          return finish(
            false,
            'variant_error',
            varRes.status,
            'hls',
            `Variant playlist returned HTTP ${varRes.status}`,
          )
        }
        const varBytes = await readUpToBytes(varRes, 256 * 1024)
        mediaPlaylistText = new TextDecoder('utf-8', { fatal: false })
          .decode(varBytes)
          .trimStart()
        mediaPlaylistUrl = varRes.url || variantUrl
        if (!mediaPlaylistText.startsWith('#EXTM3U')) {
          return finish(
            false,
            'invalid_manifest',
            varRes.status,
            'hls',
            'Variant playlist did not start with #EXTM3U',
          )
        }
      }

      const firstSegmentUrl = extractFirstHlsUri(mediaPlaylistText, mediaPlaylistUrl)
      if (!firstSegmentUrl) {
        return finish(
          false,
          'empty_playlist',
          res.status,
          'hls',
          'Media playlist contained no segments',
        )
      }

      if (opts.checkSegment) {
        const segCheck = await checkSegmentReachable(firstSegmentUrl, headers, opts.timeoutMs)
        if (!segCheck.ok) {
          return finish(false, 'segment_error', segCheck.status, 'hls', segCheck.detail)
        }
      }

      if (opts.useFfprobe) {
        const probe = await runFfprobe(stream, opts.timeoutMs)
        if (!probe.ok) {
          return finish(false, 'ffprobe_failed', res.status, 'hls', probe.detail)
        }
        return finish(true, 'ok', res.status, 'hls', '', {
          video_codec: probe.videoCodec,
          audio_codec: probe.audioCodec,
          resolution: probe.resolution,
        })
      }

      return finish(true, 'ok', res.status, 'hls', '')
    }

    // Direct non-HLS stream (MPEG-TS / MP4 / Icecast / etc.): confirm bytes flow
    const sample = await readUpToBytes(res, 2048)
    if (sample.byteLength === 0) {
      return finish(false, 'empty_stream', res.status, 'direct', 'Stream returned 0 bytes')
    }

    if (opts.useFfprobe) {
      const probe = await runFfprobe(stream, opts.timeoutMs)
      if (!probe.ok) {
        return finish(false, 'ffprobe_failed', res.status, 'direct', probe.detail)
      }
      return finish(true, 'ok', res.status, 'direct', '', {
        video_codec: probe.videoCodec,
        audio_codec: probe.audioCodec,
        resolution: probe.resolution,
      })
    }

    return finish(true, 'ok', res.status, 'direct', '')
  } catch (err: unknown) {
    const isTimeout =
      err instanceof Error &&
      (err.name === 'TimeoutError' ||
        err.name === 'AbortError' ||
        err.message.toLowerCase().includes('timeout'))
    const causeMsg =
      err instanceof Error
        ? ((err as Error & { cause?: { code?: string; message?: string } }).cause?.code ||
            (err as Error & { cause?: { message?: string } }).cause?.message ||
            err.message)
        : String(err)
    return finish(false, isTimeout ? 'timeout' : 'network_error', null, 'unknown', causeMsg)
  }
}

async function runWorkerPool<T, R>(
  items: T[],
  concurrency: number,
  worker: (item: T, index: number) => Promise<R>,
): Promise<R[]> {
  const results = new Array<R>(items.length)
  let nextIndex = 0

  async function runWorker() {
    while (true) {
      const current = nextIndex++
      if (current >= items.length) return
      results[current] = await worker(items[current], current)
    }
  }

  const workerCount = Math.min(concurrency, Math.max(1, items.length))
  await Promise.all(Array.from({ length: workerCount }, () => runWorker()))
  return results
}

async function loadJson<T>(filePath: string | null, url: string): Promise<T> {
  if (filePath) {
    const raw = await readFile(filePath, 'utf-8')
    return JSON.parse(raw) as T
  }
  const res = await fetch(url, {
    headers: { 'User-Agent': DEFAULT_USER_AGENT },
    signal: AbortSignal.timeout(30_000),
  })
  if (!res.ok) {
    throw new Error(`Failed to fetch ${url}: HTTP ${res.status}`)
  }
  return (await res.json()) as T
}

function printUsage() {
  console.log(`Usage: node scripts/validate_public_iptv.ts [options]

Validates iptv-org public streams and writes valid/invalid lists in both JSON and CSV.

Options:
  --country CODES       Comma-separated ISO country codes (e.g. US,CA,GB)
  --category IDS        Comma-separated category IDs (e.g. news,sports)
  --limit N             Only check the first N matching streams
  --concurrency N       Concurrent checks (default: 50, or 15 with --ffprobe)
  --timeout MS          Per-request timeout in ms (default: 8000)
  --ffprobe             Also probe passing streams with ffprobe (codec/resolution)
  --skip-segment        Skip verifying the first HLS segment (manifest check only)
  --out-dir DIR         Output directory (default: iptv-validation-output)
  --streams-file PATH   Read streams JSON from local file instead of iptv-org API
  --channels-file PATH  Read channels JSON from local file instead of iptv-org API
  --streams-url URL     Override streams.json URL
  --channels-url URL    Override channels.json URL
  -h, --help            Show this help message`)
}

async function main() {
  const opts = parseCliArgs(process.argv.slice(2))
  if (opts.help) {
    printUsage()
    return
  }

  const startedAt = Date.now()
  console.error('Loading iptv-org streams and channels metadata...')
  const [rawStreams, rawChannels] = await Promise.all([
    loadJson<RawIptvStream[]>(opts.streamsFile, opts.streamsUrl),
    loadJson<RawIptvChannel[]>(opts.channelsFile, opts.channelsUrl),
  ])

  const streams = enrichAndFilterStreams(rawStreams, rawChannels, opts)
  console.error(
    `Validating ${streams.length} streams (concurrency=${opts.concurrency}, timeout=${opts.timeoutMs}ms, segment=${opts.checkSegment}, ffprobe=${opts.useFfprobe})...`,
  )

  let completed = 0
  let validCount = 0
  const results = await runWorkerPool(streams, opts.concurrency, async (stream) => {
    const res = await validateStream(stream, opts)
    completed++
    if (res.valid) validCount++
    if (completed % 50 === 0 || completed === streams.length) {
      const pct = ((completed / Math.max(1, streams.length)) * 100).toFixed(1)
      console.error(`Progress: ${completed}/${streams.length} (${pct}%) — valid: ${validCount}`)
    }
    return res
  })

  const valid = results.filter((r) => r.valid)
  const invalid = results.filter((r) => !r.valid)

  const reasonCounts: Record<string, number> = {}
  for (const r of results) {
    reasonCounts[r.reason] = (reasonCounts[r.reason] ?? 0) + 1
  }

  const outDir = resolve(opts.outDir)
  await mkdir(outDir, { recursive: true })

  const summary = {
    checked_at: new Date().toISOString(),
    duration_ms: Date.now() - startedAt,
    mode: opts.useFfprobe ? 'http+hls+ffprobe' : opts.checkSegment ? 'http+hls+segment' : 'http+hls',
    filters: {
      countries: opts.countries,
      categories: opts.categories,
      limit: opts.limit,
    },
    total: results.length,
    valid: valid.length,
    invalid: invalid.length,
    reasons: reasonCounts,
  }

  await Promise.all([
    writeFile(resolve(outDir, 'valid.json'), JSON.stringify(valid, null, 2) + '\n', 'utf-8'),
    writeFile(resolve(outDir, 'invalid.json'), JSON.stringify(invalid, null, 2) + '\n', 'utf-8'),
    writeFile(resolve(outDir, 'valid.csv'), toCsv(valid), 'utf-8'),
    writeFile(resolve(outDir, 'invalid.csv'), toCsv(invalid), 'utf-8'),
    writeFile(resolve(outDir, 'summary.json'), JSON.stringify(summary, null, 2) + '\n', 'utf-8'),
  ])

  console.log('\nValidation complete:')
  console.log(`  Total:   ${summary.total}`)
  console.log(`  Valid:   ${summary.valid}`)
  console.log(`  Invalid: ${summary.invalid}`)
  console.log(`  Time:    ${(summary.duration_ms / 1000).toFixed(1)}s`)
  console.log('  Breakdown by reason:')
  for (const [reason, count] of Object.entries(reasonCounts).sort((a, b) => b[1] - a[1])) {
    console.log(`    ${reason}: ${count}`)
  }
  console.log(`  Output written to: ${outDir}`)
}

const isDirectRun =
  process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href

if (isDirectRun) {
  main().catch((err) => {
    console.error('Validation failed:', err instanceof Error ? err.message : err)
    process.exit(1)
  })
}
