<script setup lang="ts">
import { ref, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../composables/useToast'
import { errorMessage } from '../utils'

const toast = useToast()

// scope: '' = 全局, 否则 = 方案 id
const props = defineProps<{
  scope?: string
}>()

const pageSize = ref(6)
const isCustom = ref(false)
const dirty = ref(false)
let mounted = true

async function load() {
  try {
    const data = await invoke<{ page_size: number; is_custom: boolean }>('get_page_size_detail', {
      scope: props.scope || null,
    })
    if (!mounted) return
    pageSize.value = data.page_size
    isCustom.value = data.is_custom
    dirty.value = false
  } catch (e) {
    if (!mounted) return
    toast.error(`加载候选词数失败: ${errorMessage(e)}`)
  }
}

async function save() {
  try {
    await invoke('save_page_size', { scope: props.scope || null, pageSize: pageSize.value })
    toast.success(props.scope ? `已保存到 ${props.scope}.custom.yaml` : '已保存')
    await load()
  } catch (e) {
    toast.error(`保存失败: ${errorMessage(e)}`)
  }
}

watch(() => props.scope, load, { immediate: true })

onUnmounted(() => {
  mounted = false
})

defineExpose({ save })
</script>

<template>
  <div class="page-size-settings">
    <div class="scope-field" :class="{ 'is-default': !isCustom && !dirty }">
      <label class="scope-label">候选词数</label>
      <input v-model.number="pageSize" type="number" min="1" max="20" @input="dirty = true" />
      <span v-if="isCustom || dirty" class="badge badge-custom">自定义</span>
      <span v-else class="badge badge-default">默认</span>
      <p v-if="!isCustom && !dirty" class="hint">
        此方案未自定义候选词数，使用全局默认值。修改后保存将写入
        {{ scope ? `${scope}.custom.yaml` : 'default.custom.yaml' }}。
      </p>
    </div>
  </div>
</template>

<style scoped>
.page-size-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 600px;
}

.scope-field {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
}

.scope-field.is-default {
  opacity: 0.7;
}

.scope-label {
  font-size: 14px;
  color: var(--color-text-secondary);
  min-width: 70px;
}

.scope-field input {
  width: 80px;
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
}

.badge-custom {
  background: var(--color-accent);
  color: white;
}

.badge-default {
  background: var(--color-bg-tertiary);
  color: var(--color-text-tertiary);
}

.hint {
  font-size: 12px;
  color: var(--color-text-tertiary);
}
</style>
