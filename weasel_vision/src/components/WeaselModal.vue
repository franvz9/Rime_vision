<script setup lang="ts">
/**
 * Shared Modal Component
 *
 * Provides consistent modal overlay behavior across all components.
 *
 * Props:
 *   show: boolean - controls visibility
 *   title: string - modal title
 *   wide: boolean - use wider layout (600px vs 400px)
 *
 * Events:
 *   @close - emitted when clicking overlay, pressing Escape, or clicking close button
 *
 * Slots:
 *   default: modal body content
 *   actions: footer actions area (typically buttons)
 */
import { ref, watch, onUnmounted } from 'vue'

const props = defineProps<{
  show: boolean
  title?: string
  wide?: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const dialogRef = ref<HTMLElement | null>(null)
let previousFocus: HTMLElement | null = null

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close')
    return
  }
  // Focus trap: Tab / Shift+Tab cycle within the dialog
  if (e.key === 'Tab' && dialogRef.value) {
    const focusable = dialogRef.value.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
    )
    if (focusable.length === 0) return
    const first = focusable[0]
    const last = focusable[focusable.length - 1]
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault()
      last.focus()
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault()
      first.focus()
    }
  }
}

function lockBodyScroll(locked: boolean) {
  document.body.style.overflow = locked ? 'hidden' : ''
}

watch(
  () => props.show,
  (visible) => {
    lockBodyScroll(visible)
    if (visible) {
      previousFocus = document.activeElement as HTMLElement | null
      // Focus the dialog after it renders
      requestAnimationFrame(() => {
        dialogRef.value?.focus()
        // If there's a focusable element inside, focus the first one
        const firstFocusable = dialogRef.value?.querySelector<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
        )
        firstFocusable?.focus()
      })
      document.addEventListener('keydown', handleKeydown)
    } else {
      document.removeEventListener('keydown', handleKeydown)
      previousFocus?.focus()
      previousFocus = null
    }
  },
)

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  lockBodyScroll(false)
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="show"
      ref="dialogRef"
      role="dialog"
      aria-modal="true"
      :aria-labelledby="title ? 'wv-modal-title' : undefined"
      class="wv-modal-overlay"
      tabindex="-1"
      @click.self="emit('close')"
    >
      <div :class="['wv-modal', { 'wv-modal-wide': wide }]">
        <h3 v-if="title" id="wv-modal-title">{{ title }}</h3>
        <slot />
        <div v-if="$slots.actions" class="wv-modal-actions">
          <slot name="actions" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.wv-modal-overlay:focus {
  outline: none;
}
/* All other modal styles provided by shared.css (.wv-modal-*) */
</style>
