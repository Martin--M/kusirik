import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import {
  pickLiveFormat,
  buildLiveUrl,
  buildMovieUrl,
  buildEpisodeUrl,
  buildApiUrl,
  buildCatchupUrl,
} from '../src/lib/url-builder.ts'

const PARAMS = {
  serverUrl: 'http://iptv.example.com:8080',
  username: 'alice',
  password: 'secret',
}

describe('url-builder', () => {
  it('picks live stream format honoring valid override and precedence', () => {
    assert.equal(pickLiveFormat(['m3u8', 'ts'], 'm3u8'), 'm3u8')
    // Ignores override not supported by provider
    assert.equal(pickLiveFormat(['m3u8', 'ts'], 'rtmp'), 'ts')
    // Prefers ts over m3u8
    assert.equal(pickLiveFormat(['m3u8', 'ts']), 'ts')
    // Falls back to m3u8 when ts is absent
    assert.equal(pickLiveFormat(['rtmp', 'm3u8']), 'm3u8')
    // Falls back to first allowed format, then ts when empty
    assert.equal(pickLiveFormat(['rtmp']), 'rtmp')
    assert.equal(pickLiveFormat([]), 'ts')
  })

  it('constructs live, movie, episode, API, and catchup URLs', () => {
    assert.equal(
      buildLiveUrl(PARAMS, 101, 'ts'),
      'http://iptv.example.com:8080/live/alice/secret/101.ts',
    )
    assert.equal(
      buildMovieUrl(PARAMS, 202, 'mkv'),
      'http://iptv.example.com:8080/movie/alice/secret/202.mkv',
    )
    assert.equal(
      buildEpisodeUrl(PARAMS, 303, 'mp4'),
      'http://iptv.example.com:8080/series/alice/secret/303.mp4',
    )
    assert.equal(
      buildApiUrl(PARAMS),
      'http://iptv.example.com:8080/player_api.php?username=alice&password=secret',
    )
    assert.equal(
      buildApiUrl(PARAMS, 'get_vod_info', { vod_id: 202 }),
      'http://iptv.example.com:8080/player_api.php?username=alice&password=secret&action=get_vod_info&vod_id=202',
    )
    assert.equal(
      buildCatchupUrl(PARAMS, 42, '2026-11-21:08-20', 120),
      'http://iptv.example.com:8080/streaming/timeshift.php?username=alice&password=secret&stream=42&start=2026-11-21%3A08-20&duration=120',
    )
  })
})
