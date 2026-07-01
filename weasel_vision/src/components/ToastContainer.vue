<script setup lang="ts">
import { useToast } from '../composables/useToast'

const { toasts, dismiss } = useToast()
</script>

<template>
  <Teleport to="body">
    <div class="toast-container" aria-live="polite" aria-label="Notifications">
      <TransitionGroup name="toast">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          :class="['toast', `toast-${toast.type}`, { 'toast-exit': !toast.visible }]"
          role="alert"
        >
          <span class="toast-icon">
            <template v-if="toast.type === 'error'">✕</template>
            <template v-else-if="toast.type === 'success'">✓</template>
            <template v-else-if="toast.type === 'warning'">⚠</template>
            <template v-else>ℹ</template>
          </span>
          <span class="toast-message">{{ toast.message }}</span>
          <button
            class="toast-close"
            @click="dismiss(toast.id)"
            aria-label="Dismiss notification"
          >✕</button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 10000;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 280px;
  max-width: 420px;
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 14px;
  line-height: 1.4;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  pointer-events: auto;
  backdrop-filter: blur(12px);
}

.toast-error {
  background: color-mix(in srgb, var(--color-danger) 95%, transparent);
  color: #fff;
}

.toast-success {
  background: color-mix(in srgb, var(--color-success) 90%, transparent);
  color: #fff;
}

.toast-warning {
  background: color-mix(in srgb, var(--color-warning) 90%, transparent);
  color: #1d1d1f;
}

.toast-info {
  background: color-mix(in srgb, var(--color-accent) 90%, transparent);
  color: #fff;
}

.toast-icon {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: bold;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.25);
}

.toast-warning .toast-icon {
  background: rgba(0, 0, 0, 0.15);
}

.toast-message {
  flex: 1;
}

.toast-close {
  flex-shrink: 0;
  background: none;
  border: none;
  cursor: pointer;
  color: inherit;
  opacity: 0.7;
  font-size: 14px;
  padding: 2px;
  line-height: 1;
}

.toast-close:hover {
  opacity: 1;
}

/* Transition animations */
.toast-enter-active {
  transition: all 0.3s ease-out;
}

.toast-leave-active {
  transition: all 0.25s ease-in;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(50px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(50px);
}
</style>
