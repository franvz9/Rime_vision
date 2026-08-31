<script setup lang="ts">
defineProps<{
  label: string
  value: string | number
  isCustom: boolean
}>()
const emit = defineEmits<{
  (e: 'update', v: string | number): void
  (e: 'restore'): void
}>()
</script>

<template>
  <div class="scope-field" :class="{ 'is-default': !isCustom }">
    <label class="scope-label">{{ label }}</label>
    <input :value="value" @input="emit('update', ($event.target as HTMLInputElement).value)" />
    <span v-if="isCustom" class="badge badge-custom">自定义</span>
    <span v-else class="badge badge-default">默认</span>
    <button v-if="isCustom" class="btn-restore" @click="emit('restore')">恢复默认</button>
  </div>
</template>

<style scoped>
.scope-field {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
}

.scope-field.is-default {
  background: var(--color-bg-tertiary);
}

.scope-label {
  font-size: 14px;
  color: var(--color-text-secondary);
  min-width: 90px;
}

.scope-field input {
  flex: 1;
  padding: 6px 8px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 14px;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
}

.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
  white-space: nowrap;
}

.badge-custom {
  background: var(--color-accent);
  color: white;
}

.badge-default {
  background: var(--color-bg-active);
  color: var(--color-text-tertiary);
}

.btn-restore {
  padding: 4px 10px;
  border: 1px solid var(--color-border-dark);
  background: var(--color-bg-secondary);
  color: var(--color-text-secondary);
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
}

.btn-restore:hover {
  background: var(--color-bg-hover);
}
</style>
