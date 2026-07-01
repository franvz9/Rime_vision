/**
 * Shared utility functions for WeaselVision
 */

/** Format byte size to human-readable string */
export function formatSize(bytes: number): string {
  if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${bytes} B`
}

// ── Color Conversion Utilities ──────────────────────────────────────────

export interface RgbColor {
  r: number
  g: number
  b: number
  a: number
}

/**
 * Parse a hex color string (Rime or CSS format) to RGB.
 * Supports: 0xAABBGGRR, 0xBBGGRR, and #RRGGBB formats.
 */
export function hexToRgb(hex: string): RgbColor | null {
  if (!hex) return null
  const cleaned = hex.replace(' ', '')

  // Rime 0xAABBGGRR format (8 digits)
  const match8 = cleaned.match(
    /^0x([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$/,
  )
  if (match8) {
    return {
      a: parseInt(match8[1], 16),
      b: parseInt(match8[2], 16),
      g: parseInt(match8[3], 16),
      r: parseInt(match8[4], 16),
    }
  }

  // Rime 0xBBGGRR format (6 digits)
  const match6 = cleaned.match(/^0x([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$/)
  if (match6) {
    return {
      b: parseInt(match6[1], 16),
      g: parseInt(match6[2], 16),
      r: parseInt(match6[3], 16),
      a: 255,
    }
  }

  // CSS #RRGGBB format
  const matchCss = cleaned.match(/^#([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$/)
  if (matchCss) {
    return {
      r: parseInt(matchCss[1], 16),
      g: parseInt(matchCss[2], 16),
      b: parseInt(matchCss[3], 16),
      a: 255,
    }
  }

  return null
}

/**
 * Convert RGB to CSS hex string.
 */
export function rgbToHex(c: { r: number; g: number; b: number } | null): string {
  if (!c) return '#000000'
  return `#${c.r.toString(16).padStart(2, '0')}${c.g.toString(16).padStart(2, '0')}${c.b.toString(16).padStart(2, '0')}`
}

/**
 * Convert CSS hex color (#RRGGBB) to Rime hex format (0xBBGGRR or 0xAABBGGRR).
 */
export function hexToRimeHex(hex: string, a: number = 255): string {
  if (!/^#[A-Fa-f0-9]{6}$/.test(hex)) {
    console.warn(`Invalid hex color: ${hex}`)
    return '0x000000'
  }
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  if (a < 255) {
    return `0x${a.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${r.toString(16).padStart(2, '0')}`
  }
  return `0x${b.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${r.toString(16).padStart(2, '0')}`
}

/**
 * Human-readable error message from an unknown error value.
 */
export function errorMessage(e: unknown): string {
  if (e === null || e === undefined) return '未知错误'
  if (typeof e === 'string') return e
  if (e instanceof Error) return e.message
  return String(e)
}

// ── Event Bus ───────────────────────────────────────────────────────────

export interface PendingDeleteDetail {
  delete_type: string
  identifier: string
  label?: string
}

/** Event name constants and their payload types */
export const BusEvents = {
  ADD_PENDING_DELETE: 'add-pending-delete' as const,
  REMOVE_PENDING_DELETE: 'remove-pending-delete' as const,
  DEPLOY_COMPLETE: 'deploy-complete' as const,
  NAVIGATE_TO_SYNC_SETTINGS: 'navigate-to-sync-settings' as const,
} as const

export type BusEventMap = {
  [BusEvents.ADD_PENDING_DELETE]: PendingDeleteDetail
  [BusEvents.REMOVE_PENDING_DELETE]: PendingDeleteDetail
  [BusEvents.DEPLOY_COMPLETE]: null
  [BusEvents.NAVIGATE_TO_SYNC_SETTINGS]: void
}

/**
 * Type-safe event dispatch via window CustomEvent.
 * Use instead of raw `window.dispatchEvent(new CustomEvent(...))`.
 *
 * Example:
 *   emitBusEvent(BusEvents.ADD_PENDING_DELETE, { delete_type: 'schema', identifier: 'abc' })
 */
export function emitBusEvent<K extends keyof BusEventMap>(name: K, detail: BusEventMap[K]) {
  window.dispatchEvent(new CustomEvent(name, { detail }))
}

/**
 * Type-safe event listener with automatic cleanup.
 * Returns an unsubscribe function.
 */
export function onBusEvent<K extends keyof BusEventMap>(
  name: K,
  handler: (detail: BusEventMap[K]) => void,
): () => void {
  const listener = (e: Event) => {
    handler((e as CustomEvent<BusEventMap[K]>).detail)
  }
  window.addEventListener(name, listener)
  return () => window.removeEventListener(name, listener)
}
