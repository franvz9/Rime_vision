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
 *   @close - emitted when clicking overlay or close button
 *
 * Slots:
 *   default: modal body content
 *   actions: footer actions area (typically buttons)
 */
defineProps<{
  show: boolean
  title?: string
  wide?: boolean
}>()

const emit = defineEmits<{
  close: []
}>()
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="emit('close')">
      <div :class="['modal', { 'modal-wide': wide }]">
        <h3 v-if="title">{{ title }}</h3>
        <slot />
        <div v-if="$slots.actions" class="modal-actions">
          <slot name="actions" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-bg-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-overlay.z-high {
  z-index: 1000;
}

.modal {
  background: var(--color-bg-modal);
  border-radius: 12px;
  padding: 24px;
  width: 400px;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: var(--shadow-lg);
  color: var(--color-text-primary);
}

.modal-wide {
  width: 600px;
}

.modal h3 {
  margin-bottom: 12px;
  font-size: 16px;
  color: var(--color-text-primary);
}

.modal h4 {
  margin-bottom: 8px;
  font-size: 14px;
  color: var(--color-text-secondary);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}
</style>
