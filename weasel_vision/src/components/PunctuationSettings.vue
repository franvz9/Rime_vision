<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../composables/useToast'
import { errorMessage } from '../utils'
import WeaselModal from './WeaselModal.vue'
import type { PunctRule } from '../../src-tauri/bindings/PunctRule'
import type { PunctuationProfile } from '../../src-tauri/bindings/PunctuationProfile'

const props = defineProps<{
  /** 作用域: '' 或 undefined = 全局, 否则为方案 id */
  scope?: string
}>()

const toast = useToast()

const data = ref<PunctuationProfile | null>(null)
const activeTab = ref<'half' | 'full'>('half')
const editingRule = ref<PunctRule | null>(null)
const showEditor = ref(false)
const editType = ref<'commit' | 'pair' | 'list'>('commit')
const originalKey = ref('')

/** 原本自定义（custom 文件中显式定义）的键集合 */
const customKeys = ref<Set<string>>(new Set())
/** 用户本次会话新增/修改的键集合（差异写入依据） */
const dirtyKeys = ref<Set<string>>(new Set())

const isCustomKey = (key: string) => customKeys.value.has(key) || dirtyKeys.value.has(key)

watch(
  editingRule,
  (rule) => {
    if (rule) {
      if (rule.pair?.length) editType.value = 'pair'
      else if (rule.list?.length) editType.value = 'list'
      else editType.value = 'commit'
    }
  },
  { immediate: true },
)

const currentRules = computed(() => {
  if (!data.value) return []
  return activeTab.value === 'half' ? data.value.half_shape : data.value.full_shape
})

let punctMounted = true

async function load() {
  try {
    const profile = await invoke<PunctuationProfile>('get_punctuation_profile', {
      scope: props.scope || null,
    })
    if (!punctMounted) return
    data.value = profile
    customKeys.value = new Set(profile.custom_keys || [])
    dirtyKeys.value = new Set()
  } catch (e) {
    if (!punctMounted) return
    toast.error(`加载标点设置失败: ${errorMessage(e)}`)
  }
}

onMounted(() => {
  load()
})

watch(() => props.scope, load)

onUnmounted(() => {
  punctMounted = false
})

defineExpose({ save })

function editRule(rule: PunctRule) {
  originalKey.value = rule.key
  editingRule.value = { ...rule, pair: [...rule.pair], list: [...rule.list] }
  showEditor.value = true
}

function addRule() {
  originalKey.value = ''
  editingRule.value = { key: '', commit: '', pair: [], list: [] }
  showEditor.value = true
}

function deleteRule(key: string) {
  if (!data.value) return
  if (activeTab.value === 'half') {
    data.value.half_shape = data.value.half_shape.filter((r) => r.key !== key)
  } else {
    data.value.full_shape = data.value.full_shape.filter((r) => r.key !== key)
  }
  // 从 custom 集合中移除该键：保存时不再包含它，等于从方案 custom 中删除
  customKeys.value.delete(key)
  dirtyKeys.value.delete(key)
}

function saveEditedRule(rule: PunctRule) {
  if (!data.value) return
  const target = activeTab.value === 'half' ? data.value.half_shape : data.value.full_shape
  const idx = originalKey.value ? target.findIndex((r) => r.key === originalKey.value) : -1
  if (idx >= 0) {
    target[idx] = rule
  } else {
    target.push(rule)
  }
  target.sort((a, b) => a.key.localeCompare(b.key))
  // 标记该键为 dirty（新增或修改 → 需写入 custom）
  dirtyKeys.value.add(rule.key)
  showEditor.value = false
  editingRule.value = null
  originalKey.value = ''
}

/**
 * 差异保存：仅将"原本自定义 + 本次修改/新增"的键传给后端。
 * 默认项（既非 custom 又未修改）不写入 custom 文件。
 */
async function save() {
  if (!data.value) return
  try {
    // 需要写入的键 = customKeys ∪ dirtyKeys
    const needWrite = new Set<string>([...customKeys.value, ...dirtyKeys.value])
    const half = data.value.half_shape.filter((r) => needWrite.has(r.key))
    const full = data.value.full_shape.filter((r) => needWrite.has(r.key))
    await invoke('save_punctuation', {
      scope: props.scope || null,
      half,
      full,
    })
    // 保存成功后，这些键成为"自定义"
    customKeys.value = new Set(needWrite)
    dirtyKeys.value = new Set()
    toast.success(props.scope ? `已保存到 ${props.scope}.custom.yaml` : '标点设置已保存')
  } catch (e) {
    toast.error(`保存标点设置失败: ${errorMessage(e)}`)
  }
}
</script>

<template>
  <div class="punctuation-settings">
    <div class="tabs">
      <button :class="['tab', { active: activeTab === 'half' }]" @click="activeTab = 'half'">
        半角标点
      </button>
      <button :class="['tab', { active: activeTab === 'full' }]" @click="activeTab = 'full'">
        全角标点
      </button>
    </div>

    <div class="list-header">
      <h3 class="list-title">{{ activeTab === 'half' ? '半角标点' : '全角标点' }}</h3>
      <button class="add-btn" @click="addRule">+ 新增</button>
    </div>

    <div class="rule-list">
      <div
        v-for="rule in currentRules"
        :key="rule.key"
        :class="['rule-item', { 'rule-default': !isCustomKey(rule.key) }]"
      >
        <span class="rule-key">{{ rule.key }}</span>
        <span class="arrow">→</span>
        <span class="rule-value">
          {{
            rule.commit ||
            (rule.pair.length
              ? `[${rule.pair.join(', ')}]`
              : rule.list.length
                ? `[${rule.list.join(', ')}]`
                : '')
          }}
        </span>
        <span class="rule-type">
          {{ rule.commit ? 'commit' : rule.pair.length ? 'pair' : 'list' }}
        </span>
        <span v-if="isCustomKey(rule.key)" class="badge badge-custom">自定义</span>
        <span v-else class="badge badge-default">默认</span>
        <span class="rule-actions">
          <button class="wv-icon-btn" @click="editRule(rule)">📝</button>
          <button class="wv-icon-btn danger" @click="deleteRule(rule.key)">🗑</button>
        </span>
      </div>
    </div>

    <!-- Edit modal -->
    <WeaselModal :show="showEditor && !!editingRule" title="编辑标点" @close="showEditor = false">
      <div class="wv-form-group">
        <label>按键:</label>
        <input v-model="editingRule!.key" maxlength="4" />
      </div>
      <div class="wv-form-group">
        <label>类型:</label>
        <div class="wv-radio-group">
          <label class="wv-radio-item">
            <input v-model="editType" type="radio" value="commit" />
            <span>直接上屏</span>
          </label>
          <label class="wv-radio-item">
            <input v-model="editType" type="radio" value="pair" />
            <span>配对输入</span>
          </label>
          <label class="wv-radio-item">
            <input v-model="editType" type="radio" value="list" />
            <span>候选列表</span>
          </label>
        </div>
      </div>
      <div v-if="editType === 'commit'" class="wv-form-group">
        <label>输出:</label>
        <input v-model="editingRule!.commit" />
      </div>
      <div v-else-if="editType === 'pair'" class="wv-form-group">
        <label>配对 (逗号分隔):</label>
        <input
          :value="editingRule!.pair.join(', ')"
          @input="
            editingRule!.pair = ($event.target as HTMLInputElement).value
              .split(',')
              .map((s) => s.trim())
          "
        />
      </div>
      <div v-else class="wv-form-group">
        <label>候选 (逗号分隔):</label>
        <input
          :value="editingRule!.list.join(', ')"
          @input="
            editingRule!.list = ($event.target as HTMLInputElement).value
              .split(',')
              .map((s) => s.trim())
          "
        />
      </div>
      <template #actions>
        <button class="wv-btn" @click="showEditor = false">取消</button>
        <button
          class="wv-btn wv-btn-primary"
          :disabled="!editingRule?.key"
          @click="saveEditedRule(editingRule!)"
        >
          保存
        </button>
      </template>
    </WeaselModal>
  </div>
</template>

<style scoped>
.punctuation-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 600px;
}

.tabs {
  display: flex;
  gap: 2px;
  border-bottom: 1px solid var(--color-border);
  width: fit-content;
}

.tab {
  padding: 6px 12px;
  border: none;
  border-bottom: 2px solid transparent;
  background: none;
  color: var(--color-text-tertiary);
  cursor: pointer;
  font-size: 13px;
  transition:
    color 0.15s,
    border-color 0.15s;
}

.tab:hover {
  color: var(--color-text-primary);
}

.tab.active {
  color: var(--color-accent);
  border-bottom-color: var(--color-accent);
  font-weight: 500;
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 4px 0 8px;
}

.list-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.add-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 5px 12px;
  border: 1px solid var(--color-border-dark);
  border-radius: 6px;
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  font-size: 13px;
  cursor: pointer;
  transition:
    background 0.15s,
    border-color 0.15s;
}

.add-btn:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.rule-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
}

.rule-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  font-size: 13px;
}

.rule-item:nth-child(even) {
  background: var(--color-bg-tertiary);
}

.rule-item.rule-default {
  opacity: 0.65;
}

.badge {
  display: inline-block;
  padding: 1px 6px;
  border-radius: 10px;
  font-size: 10px;
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

.rule-key {
  font-family: monospace;
  font-weight: 500;
  min-width: 30px;
  text-align: center;
}

.arrow {
  color: var(--color-text-tertiary);
}

.rule-value {
  flex: 1;
  font-family: monospace;
}

.rule-type {
  font-size: 11px;
  color: var(--color-text-tertiary);
  min-width: 40px;
}

.rule-actions {
  display: flex;
  gap: 4px;
}

.wv-icon-btn {
  font-size: 18px;
  padding: 4px 6px;
  line-height: 1;
}

.wv-btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
}

.wv-form-group {
  margin-bottom: 12px;
}

.wv-form-group label {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin-bottom: 4px;
}

.wv-form-group input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 14px;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
}
</style>
