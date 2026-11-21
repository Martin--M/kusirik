import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import { getCountryName } from '../src/lib/countries.ts'

describe('countries', () => {
  it('resolves known ISO country codes case-insensitively with whitespace trimmed', () => {
    assert.equal(getCountryName('US'), 'United States')
    assert.equal(getCountryName('ca'), 'Canada')
    assert.equal(getCountryName('  fr  '), 'France')
    assert.equal(getCountryName('gb'), 'United Kingdom')
  })

  it('falls back to uppercased code for unknown entries', () => {
    assert.equal(getCountryName('xx'), 'XX')
    assert.equal(getCountryName('  int '), 'INT')
  })
})
