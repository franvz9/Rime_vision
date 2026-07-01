import { describe, it, expect } from 'vitest'
import { errorMessage, formatSize, hexToRgb, rgbToHex, hexToRimeHex } from '../utils'

// ── errorMessage ─────────────────────────────────────────────────────────────

describe('errorMessage', () => {
  it('returns default message for null', () => {
    expect(errorMessage(null)).toBe('未知错误')
  })

  it('returns default message for undefined', () => {
    expect(errorMessage(undefined)).toBe('未知错误')
  })

  it('returns error.message for Error objects', () => {
    expect(errorMessage(new Error('something broke'))).toBe('something broke')
  })

  it('returns the string directly when passed a string', () => {
    expect(errorMessage('plain error string')).toBe('plain error string')
  })

  it('converts a plain object to its string representation', () => {
    const obj = { code: 'ERR', message: 'Tauri serialised error' }
    // String(obj) === '[object Object]'
    expect(errorMessage(obj)).toBe(String(obj))
  })

  it('converts a number to its string representation', () => {
    expect(errorMessage(42)).toBe('42')
  })
})

// ── formatSize ───────────────────────────────────────────────────────────────

describe('formatSize', () => {
  it('formats bytes (B)', () => {
    expect(formatSize(512)).toBe('512 B')
    expect(formatSize(0)).toBe('0 B')
  })

  it('formats kilobytes (KB)', () => {
    expect(formatSize(1024)).toBe('1.0 KB')
    expect(formatSize(1536)).toBe('1.5 KB')
  })

  it('formats megabytes (MB)', () => {
    expect(formatSize(1024 * 1024)).toBe('1.0 MB')
    expect(formatSize(2.5 * 1024 * 1024)).toBe('2.5 MB')
  })
})

// ── hexToRgb ─────────────────────────────────────────────────────────────────

describe('hexToRgb', () => {
  it('parses Rime 8-digit format (0xAABBGGRR)', () => {
    expect(hexToRgb('0xFF0000FF')).toEqual({ a: 255, b: 0, g: 0, r: 255 })
  })

  it('parses Rime 6-digit format (0xBBGGRR)', () => {
    expect(hexToRgb('0x0000FF')).toEqual({ b: 0, g: 0, r: 255, a: 255 })
  })

  it('parses CSS #RRGGBB format', () => {
    expect(hexToRgb('#FF0000')).toEqual({ r: 255, g: 0, b: 0, a: 255 })
    expect(hexToRgb('#00FF00')).toEqual({ r: 0, g: 255, b: 0, a: 255 })
    expect(hexToRgb('#0000FF')).toEqual({ r: 0, g: 0, b: 255, a: 255 })
  })

  it('returns null for empty or invalid input', () => {
    expect(hexToRgb('')).toBeNull()
    expect(hexToRgb('not-a-color')).toBeNull()
    expect(hexToRgb('#GGG')).toBeNull()
    expect(hexToRgb('0xZZ')).toBeNull()
  })

  it('handles spaces in the hex string', () => {
    // utils.ts strips a single space before parsing
    expect(hexToRgb('0x 0000FF')).toEqual({ b: 0, g: 0, r: 255, a: 255 })
  })
})

// ── rgbToHex ─────────────────────────────────────────────────────────────────

describe('rgbToHex', () => {
  it('converts RGB to CSS hex', () => {
    expect(rgbToHex({ r: 255, g: 0, b: 0 })).toBe('#ff0000')
    expect(rgbToHex({ r: 0, g: 255, b: 0 })).toBe('#00ff00')
    expect(rgbToHex({ r: 0, g: 0, b: 255 })).toBe('#0000ff')
  })

  it('returns #000000 for null input', () => {
    expect(rgbToHex(null)).toBe('#000000')
  })

  it('pads single-digit hex values', () => {
    expect(rgbToHex({ r: 1, g: 2, b: 3 })).toBe('#010203')
  })
})

// ── hexToRimeHex ──────────────────────────────────────────────────────────────

describe('hexToRimeHex', () => {
  it('converts CSS hex to Rime 6-digit format (0xBBGGRR) when alpha=255', () => {
    // #FF0000 → R=FF, G=00, B=00 → 0x0000FF
    expect(hexToRimeHex('#FF0000')).toBe('0x0000ff')
  })

  it('converts CSS hex to Rime 8-digit format (0xAABBGGRR) when alpha<255', () => {
    // #FF0000, alpha=128(0x80) → 0x800000ff
    expect(hexToRimeHex('#FF0000', 128)).toBe('0x800000ff')
  })

  it('returns 0x000000 for invalid CSS hex', () => {
    expect(hexToRimeHex('not-a-color')).toBe('0x000000')
    expect(hexToRimeHex('#FFF')).toBe('0x000000')
    expect(hexToRimeHex('')).toBe('0x000000')
  })

  it('handles all-zero color', () => {
    expect(hexToRimeHex('#000000')).toBe('0x000000')
  })
})
