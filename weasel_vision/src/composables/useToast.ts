/**
 * Global Toast Notification System
 *
 * Provides a reactive toast notification system usable by any component.
 * Uses module-level reactive state so all components share the same toast queue.
 *
 * Usage:
 *   import { useToast } from '@/composables/useToast'
 *   const toast = useToast()
 *   toast.error('Failed to save settings')
 *   toast.success('Settings saved')
 *   toast.warning('Please review your changes')
 *   toast.info('Deploy in progress...')
 */

import { ref } from 'vue'

export interface Toast {
  id: number
  type: 'error' | 'success' | 'warning' | 'info'
  message: string
  visible: boolean
  /** Timer ID for auto-dismiss, cleared on manual dismiss to avoid double-fire */
  timer: ReturnType<typeof setTimeout> | null
  /** Timer ID for removal-after-animation, cleared if toast is already removed */
  removeTimer: ReturnType<typeof setTimeout> | null
}

let nextId = 0
const toasts = ref<Toast[]>([])

function addToast(type: Toast['type'], message: string, duration = 4000) {
  const id = nextId++
  const toast: Toast = {
    id,
    type,
    message,
    visible: true,
    timer: null,
    removeTimer: null,
  }
  toasts.value.push(toast)

  // Auto-dismiss after duration
  toast.timer = setTimeout(() => {
    dismissToast(id)
  }, duration)
}

function dismissToast(id: number) {
  const idx = toasts.value.findIndex(t => t.id === id)
  if (idx === -1) return

  const toast = toasts.value[idx]

  // Clear auto-dismiss timer to prevent double-fire
  if (toast.timer !== null) {
    clearTimeout(toast.timer)
  }

  toast.visible = false

  // Remove from array after animation
  toast.removeTimer = setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }, 300)
}

export function useToast() {
  return {
    toasts,

    /** Show an error notification (auto-dismiss: 6s) */
    error(message: string) {
      addToast('error', message, 6000)
    },

    /** Show a success notification (auto-dismiss: 3s) */
    success(message: string) {
      addToast('success', message, 3000)
    },

    /** Show a warning notification (auto-dismiss: 5s) */
    warning(message: string) {
      addToast('warning', message, 5000)
    },

    /** Show an info notification (auto-dismiss: 3s) */
    info(message: string) {
      addToast('info', message, 3000)
    },

    /** Dismiss a specific toast by id */
    dismiss(id: number) {
      dismissToast(id)
    },
  }
}
