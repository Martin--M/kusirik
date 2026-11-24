import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import { createServer } from 'node:http'
import type { AddressInfo } from 'node:net'
import {
  parseCliArgs,
  classifyHttpStatus,
  isMasterPlaylist,
  extractFirstHlsUri,
  enrichAndFilterStreams,
  toCsv,
  validateStream,
  type EnrichedStream,
  type ValidationResult,
} from '../scripts/validate_public_iptv.ts'

describe('validate_public_iptv helpers', () => {
  it('parses CLI arguments and adjusts default concurrency for --ffprobe', () => {
    const defaults = parseCliArgs([])
    assert.equal(defaults.concurrency, 50)
    assert.equal(defaults.useFfprobe, false)
    assert.equal(defaults.checkSegment, true)

    const probed = parseCliArgs(['--ffprobe', '--country', 'us, ca', '--category', 'News', '--limit', '100'])
    assert.equal(probed.useFfprobe, true)
    assert.equal(probed.concurrency, 15)
    assert.deepEqual(probed.countries, ['US', 'CA'])
    assert.deepEqual(probed.categories, ['news'])
    assert.equal(probed.limit, 100)

    const customConcurrency = parseCliArgs(['--ffprobe', '--concurrency', '8', '--skip-segment'])
    assert.equal(customConcurrency.concurrency, 8)
    assert.equal(customConcurrency.checkSegment, false)

    assert.throws(() => parseCliArgs(['--limit', '0']))
    assert.throws(() => parseCliArgs(['--unknown']))
  })

  it('classifies HTTP status codes accurately', () => {
    assert.equal(classifyHttpStatus(200), null)
    assert.equal(classifyHttpStatus(206), null)
    assert.equal(classifyHttpStatus(403), 'geo_blocked')
    assert.equal(classifyHttpStatus(451), 'geo_blocked')
    assert.equal(classifyHttpStatus(401), 'unauthorized')
    assert.equal(classifyHttpStatus(404), 'not_found')
    assert.equal(classifyHttpStatus(410), 'not_found')
    assert.equal(classifyHttpStatus(502), 'server_error')
    assert.equal(classifyHttpStatus(429), 'http_error')
  })

  it('detects master playlists and resolves relative HLS URIs', () => {
    const master = `#EXTM3U
#EXT-X-STREAM-INF:BANDWIDTH=1280000
variants/720p/index.m3u8
`
    assert.equal(isMasterPlaylist(master), true)
    assert.equal(
      extractFirstHlsUri(master, 'https://cdn.example.com/live/master.m3u8'),
      'https://cdn.example.com/live/variants/720p/index.m3u8',
    )

    const media = `#EXTM3U
#EXTINF:6.0,
seg001.ts
`
    assert.equal(isMasterPlaylist(media), false)
    assert.equal(
      extractFirstHlsUri(media, 'https://cdn.example.com/live/variants/720p/index.m3u8'),
      'https://cdn.example.com/live/variants/720p/seg001.ts',
    )
    assert.equal(extractFirstHlsUri('#EXTM3U\n#EXT-X-ENDLIST\n', 'https://cdn.example.com/a.m3u8'), null)
  })

  it('enriches and filters streams by country, category, and limit', () => {
    const channels = [
      { id: 'ch1', name: 'US News', country: 'US', categories: ['news'] },
      { id: 'ch2', name: 'CA Sports', country: 'CA', categories: ['sports'] },
    ]
    const streams = [
      { channel: 'ch1', title: 'Raw 1', url: 'https://a/1.m3u8', http_referrer: 'https://ref' },
      { channel: 'ch2', title: 'Raw 2', url: 'https://a/2.m3u8' },
    ]

    const filtered = enrichAndFilterStreams(streams, channels, {
      countries: ['US'],
      categories: ['news'],
      limit: 10,
    })
    assert.equal(filtered.length, 1)
    assert.equal(filtered[0].name, 'US News')
    assert.equal(filtered[0].referrer, 'https://ref')
  })

  it('escapes commas and quotes properly in CSV output', () => {
    const row: ValidationResult = {
      channel_id: 'ch1',
      name: 'News, "Live" HD',
      url: 'https://example.com/live.m3u8',
      country: 'US',
      categories: 'news',
      valid: true,
      reason: 'ok',
      http_status: 200,
      latency_ms: 42,
      stream_kind: 'hls',
      video_codec: 'h264',
      audio_codec: 'aac',
      resolution: '1920x1080',
      detail: '',
    }
    const csv = toCsv([row])
    const lines = csv.trim().split('\n')
    assert.equal(lines.length, 2)
    assert.ok(lines[1].includes('"News, ""Live"" HD"'))
  })

  it('validates HLS master/variant/segment and catches dead or HTML responses against a local server', async () => {
    const server = createServer((req, res) => {
      if (req.url === '/master.m3u8') {
        res.writeHead(200, { 'Content-Type': 'application/vnd.apple.mpegurl' })
        res.end('#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=800000\n/media.m3u8\n')
      } else if (req.url === '/media.m3u8') {
        res.writeHead(200, { 'Content-Type': 'application/vnd.apple.mpegurl' })
        res.end('#EXTM3U\n#EXTINF:4.0,\n/seg1.ts\n')
      } else if (req.url === '/seg1.ts') {
        res.writeHead(200, { 'Content-Type': 'video/mp2t' })
        res.end(Buffer.from([0x47, 0x40, 0x00, 0x10]))
      } else if (req.url === '/broken-seg.m3u8') {
        res.writeHead(200, { 'Content-Type': 'application/vnd.apple.mpegurl' })
        res.end('#EXTM3U\n#EXTINF:4.0,\n/missing.ts\n')
      } else if (req.url === '/html.m3u8') {
        res.writeHead(200, { 'Content-Type': 'text/html' })
        res.end('<html>blocked</html>')
      } else if (req.url === '/geo.m3u8') {
        res.writeHead(403)
        res.end()
      } else {
        res.writeHead(404)
        res.end()
      }
    })

    await new Promise<void>((resolveListen) => server.listen(0, '127.0.0.1', resolveListen))
    const { port } = server.address() as AddressInfo
    const base = `http://127.0.0.1:${port}`

    const makeStream = (path: string): EnrichedStream => ({
      channel_id: 'test',
      name: 'Test',
      url: `${base}${path}`,
      country: 'US',
      categories: ['news'],
      referrer: null,
      user_agent: null,
      quality: null,
    })

    try {
      const ok = await validateStream(makeStream('/master.m3u8'), {
        timeoutMs: 3000,
        checkSegment: true,
        useFfprobe: false,
      })
      assert.equal(ok.valid, true)
      assert.equal(ok.reason, 'ok')
      assert.equal(ok.stream_kind, 'hls')

      const brokenSeg = await validateStream(makeStream('/broken-seg.m3u8'), {
        timeoutMs: 3000,
        checkSegment: true,
        useFfprobe: false,
      })
      assert.equal(brokenSeg.valid, false)
      assert.equal(brokenSeg.reason, 'segment_error')

      const htmlPage = await validateStream(makeStream('/html.m3u8'), {
        timeoutMs: 3000,
        checkSegment: true,
        useFfprobe: false,
      })
      assert.equal(htmlPage.valid, false)
      assert.equal(htmlPage.reason, 'invalid_content_type')

      const geo = await validateStream(makeStream('/geo.m3u8'), {
        timeoutMs: 3000,
        checkSegment: true,
        useFfprobe: false,
      })
      assert.equal(geo.valid, false)
      assert.equal(geo.reason, 'geo_blocked')
    } finally {
      await new Promise<void>((resolveClose) => server.close(() => resolveClose()))
    }
  })
})
