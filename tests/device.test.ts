import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import { checkIsAndroid, checkIsDesktop } from '../src/lib/device.ts'

function withUserAgent(ua: string | undefined, fn: () => void) {
  const original = Object.getOwnPropertyDescriptor(globalThis, 'navigator')
  try {
    if (ua === undefined) {
      Object.defineProperty(globalThis, 'navigator', {
        value: undefined,
        configurable: true,
        writable: true,
      })
    } else {
      Object.defineProperty(globalThis, 'navigator', {
        value: { userAgent: ua },
        configurable: true,
        writable: true,
      })
    }
    fn()
  } finally {
    if (original) {
      Object.defineProperty(globalThis, 'navigator', original)
    }
  }
}

describe('device', () => {
  it('detects Android, iOS, desktop, and missing navigator environments', () => {
    withUserAgent('Mozilla/5.0 (Linux; Android 14; Pixel 8)', () => {
      assert.equal(checkIsAndroid(), true)
      assert.equal(checkIsDesktop(), false)
    })

    withUserAgent('Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)', () => {
      assert.equal(checkIsAndroid(), false)
      assert.equal(checkIsDesktop(), false)
    })

    withUserAgent('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36', () => {
      assert.equal(checkIsAndroid(), false)
      assert.equal(checkIsDesktop(), true)
    })

    withUserAgent(undefined, () => {
      assert.equal(checkIsAndroid(), false)
      assert.equal(checkIsDesktop(), false)
    })
  })
})
