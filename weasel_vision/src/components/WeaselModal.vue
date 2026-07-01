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
    <div v-if="show" class="wv-modal-overlay" @click.self="emit('close')">
      <div :class="['wv-modal', { 'wv-modal-wide': wide }]">
        <h3 v-if="title">{{ title }}</h3>
        <slot />
        <div v-if="$slots.actions" class="wv-modal-actions">
          <slot name="actions" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
/* All modal styles provided by shared.css (.wv-modal-*) */
</style>
