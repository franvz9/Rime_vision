<script setup lang="ts">
import { ref, computed, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../composables/useToast'
import { errorMessage } from '../utils'
import type { TranslatorSettings as TSettings } from '../../src-tauri/bindings/TranslatorSettings'

const toast = useToast()

// scope: '' = 全局, 否则 = 方案 id
const props = defineProps<{
  scope?: string
}>()

const settings = ref<TSettings>({
  enable_encoder: true,
  enable_sentence: true,
  enable_user_dict: true,
  encode_commit_history: true,
})
const isCustom = ref(false)
/** 本次会话被改动过的字段集合（差异写入依据） */
const dirtyFields = ref<Set<keyof TSettings>>(new Set())
const dirty = computed(() => dirtyFields.value.size > 0)
let mounted = true

async function load() {
  try {
    const profile = await invoke<{
      settings: TSettings
      is_custom: boolean
    }>('get_translator_profile', {
      scope: props.scope || null,
    })
    if (!mounted) return
    settings.value = { ...profile.settings }
    isCustom.value = profile.is_custom
    dirtyFields.value = new Set()
  } catch (e) {
    if (!mounted) return
    toast.error(`加载翻译器设置失败: ${errorMessage(e)}`)
  }
}

/** 标记某个字段为已改动 */
function markDirty(field: keyof TSettings) {
  dirtyFields.value.add(field)
  dirtyFields.value = new Set(dirtyFields.value)
}

async function save() {
  try {
    // 差异写入：仅传被改动字段（未被改动字段保持 None，后端不写入）
    const patch = {
      enable_encoder: dirtyFields.value.has('enable_encoder')
        ? settings.value.enable_encoder
        : null,
      enable_sentence: dirtyFields.value.has('enable_sentence')
        ? settings.value.enable_sentence
        : null,
      enable_user_dict: dirtyFields.value.has('enable_user_dict')
        ? settings.value.enable_user_dict
        : null,
      encode_commit_history: dirtyFields.value.has('encode_commit_history')
        ? settings.value.encode_commit_history
        : null,
    }
    await invoke('save_translator', { scope: props.scope || null, patch })
    toast.success(props.scope ? `已保存到 ${props.scope}.custom.yaml` : '翻译器设置已保存')
    await load()
  } catch (e) {
    toast.error(`保存翻译器设置失败: ${errorMessage(e)}`)
  }
}

watch(() => props.scope, load, { immediate: true })

onUnmounted(() => {
  mounted = false
})

defineExpose({ save })
</script>

<template>
  <div class="translator-settings">
    <div class="translator-card" :class="{ 'is-default': !isCustom && !dirty }">
      <label class="checkbox-row">
        <input
          v-model="settings.enable_encoder"
          type="checkbox"
          @change="markDirty('enable_encoder')"
        />
        <span>
          <strong>启用自动造词</strong>
          <small class="sub">(enable_encoder) 输入时自动学习并造词</small>
        </span>
      </label>
      <label class="checkbox-row">
        <input
          v-model="settings.enable_sentence"
          type="checkbox"
          @change="markDirty('enable_sentence')"
        />
        <span>
          <strong>启用自动句子输入</strong>
          <small class="sub">(enable_sentence) 自动对短语进行编码，形成句子</small>
        </span>
      </label>
      <label class="checkbox-row">
        <input
          v-model="settings.enable_user_dict"
          type="checkbox"
          @change="markDirty('enable_user_dict')"
        />
        <span>
          <strong>启用用户词典</strong>
          <small class="sub">(enable_user_dict) 允许使用用户词典</small>
        </span>
      </label>
      <label class="checkbox-row">
        <input
          v-model="settings.encode_commit_history"
          type="checkbox"
          @change="markDirty('encode_commit_history')"
        />
        <span>
          <strong>自动编码上屏词语</strong>
          <small class="sub">(encode_commit_history) 对已上屏的词语进行编码</small>
        </span>
      </label>

      <div class="badge-row">
        <span v-if="isCustom || dirty" class="badge badge-custom">自定义</span>
        <span v-else class="badge badge-default">默认</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.translator-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 600px;
}

.translator-card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 16px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
}

.translator-card.is-default {
  opacity: 0.8;
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 0;
  cursor: pointer;
}

.checkbox-row input[type='checkbox'] {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.checkbox-row span {
  display: flex;
  flex-direction: column;
  font-size: 13px;
}

.checkbox-row .sub {
  font-size: 11px;
  color: var(--color-text-tertiary);
  margin-top: 2px;
}

.badge-row {
  margin-top: 8px;
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
</style>
