<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['changed'])

interface PunctRule {
  key: string
  commit: string
  pair: string[]
  list: string[]
}

interface PunctuationData {
  half_shape: PunctRule[]
  full_shape: PunctRule[]
}

const data = ref<PunctuationData | null>(null)
const activeTab = ref<'half' | 'full'>('half')
const editingRule = ref<PunctRule | null>(null)
const showEditor = ref(false)
const showSaved = ref(false)
const editType = ref<'commit' | 'pair' | 'list'>('commit')
const originalKey = ref('')

watch(editingRule, (rule) => {
  if (rule) {
    if (rule.pair?.length) editType.value = 'pair'
    else if (rule.list?.length) editType.value = 'list'
    else editType.value = 'commit'
  }
}, { immediate: true })

const currentRules = computed(() => {
  if (!data.value) return []
  return activeTab.value === 'half' ? data.value.half_shape : data.value.full_shape
})

onMounted(async () => {
  try {
    data.value = await invoke('get_punctuation')
  } catch (e) {
    console.error('Failed to load punctuation:', e)
  }
})

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
  emit('changed')
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
  showEditor.value = false
  editingRule.value = null
  originalKey.value = ''
  emit('changed')
}

async function save() {
  if (!data.value) return
  try {
    await invoke('save_punctuation', {
      half: data.value.half_shape,
      full: data.value.full_shape,
    })
    showSaved.value = true
    setTimeout(() => { showSaved.value = false }, 2000)
    emit('changed')
  } catch (e) {
    console.error('Failed to save punctuation:', e)
  }
}


</script>

<template>
  <div class="punctuation-settings">
    <div class="tabs">
      <button :class="['tab', { active: activeTab === 'half' }]" @click="activeTab = 'half'">半角标点</button>
      <button :class="['tab', { active: activeTab === 'full' }]" @click="activeTab = 'full'">全角标点</button>
      <button class="btn-add" @click="addRule">+</button>
    </div>

    <div class="rule-list">
      <div v-for="rule in currentRules" :key="rule.key" class="rule-item">
        <span class="rule-key">{{ rule.key }}</span>
        <span class="arrow">→</span>
        <span class="rule-value">
          {{ rule.commit || (rule.pair.length ? `[${rule.pair.join(', ')}]` : rule.list.length ? `[${rule.list.join(', ')}]` : '') }}
        </span>
        <span class="rule-type">
          {{ rule.commit ? 'commit' : rule.pair.length ? 'pair' : 'list' }}
        </span>
        <span class="rule-actions">
          <button class="icon-btn" @click="editRule(rule)">✏</button>
          <button class="icon-btn danger" @click="deleteRule(rule.key)">🗑</button>
        </span>
      </div>
    </div>

    <div class="actions">
      <span v-if="showSaved" class="saved-hint">已保存</span>
      <button class="btn btn-primary" @click="save">保存到 default.custom.yaml</button>
    </div>

    <!-- Edit modal -->
    <div v-if="showEditor" class="modal-overlay" @click.self="showEditor = false">
      <div class="modal">
        <h3>编辑标点</h3>
        <div class="form-group">
          <label>按键:</label>
          <input v-model="editingRule!.key" maxlength="4" />
        </div>
        <div class="form-group">
          <label>类型:</label>
          <div class="radio-group">
            <label><input type="radio" value="commit" v-model="editType" /> 直接上屏</label>
            <label><input type="radio" value="pair" v-model="editType" /> 配对输入</label>
            <label><input type="radio" value="list" v-model="editType" /> 候选列表</label>
          </div>
        </div>
        <div class="form-group" v-if="editType === 'commit'">
          <label>输出:</label>
          <input v-model="editingRule!.commit" />
        </div>
        <div class="form-group" v-else-if="editType === 'pair'">
          <label>配对 (逗号分隔):</label>
          <input :value="editingRule!.pair.join(', ')" @input="editingRule!.pair = ($event.target as HTMLInputElement).value.split(',').map(s => s.trim())" />
        </div>
        <div class="form-group" v-else>
          <label>候选 (逗号分隔):</label>
          <input :value="editingRule!.list.join(', ')" @input="editingRule!.list = ($event.target as HTMLInputElement).value.split(',').map(s => s.trim())" />
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showEditor = false">取消</button>
          <button class="btn btn-primary" @click="saveEditedRule(editingRule!)" :disabled="!editingRule?.key">保存</button>
        </div>
      </div>
    </div>
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
  gap: 4px;
  background: #e9e9eb;
  padding: 4px;
  border-radius: 8px;
  width: fit-content;
}

.tab {
  padding: 6px 16px;
  border: none;
  background: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.tab.active {
  background: white;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.btn-add {
  width: 32px;
  height: 32px;
  border: 1px dashed #ccc;
  background: white;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
  margin-left: 8px;
}

.rule-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 400px;
  overflow-y: auto;
  background: white;
  border: 1px solid #e5e5e5;
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
  background: #f9f9f9;
}

.rule-key {
  font-family: monospace;
  font-weight: 500;
  min-width: 30px;
  text-align: center;
}

.arrow {
  color: #999;
}

.rule-value {
  flex: 1;
  font-family: monospace;
}

.rule-type {
  font-size: 11px;
  color: #999;
  min-width: 40px;
}

.rule-actions {
  display: flex;
  gap: 4px;
}

.icon-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  padding: 2px 4px;
}

.icon-btn.danger {
  color: #ff3b30;
}

.actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn {
  padding: 8px 16px;
  border: 1px solid #ddd;
  background: white;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.btn-primary {
  background: #007aff;
  color: white;
  border: none;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.saved-hint {
  color: #34c759;
  font-size: 13px;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: white;
  border-radius: 12px;
  padding: 24px;
  width: 400px;
}

.modal h3 {
  margin-bottom: 16px;
}

.form-group {
  margin-bottom: 12px;
}

.form-group label {
  display: block;
  font-size: 13px;
  color: #666;
  margin-bottom: 4px;
}

.form-group input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.radio-group {
  display: flex;
  gap: 16px;
}

.radio-group label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  cursor: pointer;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}
</style>
