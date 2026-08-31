<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import DeployNotice from './DeployNotice.vue'
import PageSizeSettings from './PageSizeSettings.vue'
import PunctuationSettings from './PunctuationSettings.vue'
import KeybindingEditor from './KeybindingEditor.vue'

const schemas = ref<string[]>([])
const selectedSchema = ref('')
const activeTab = ref<'punctuation' | 'keybinding' | 'candidate'>('punctuation')
const showGlobalRef = ref(false)
const globalRef = ref<{ page_size: number } | null>(null)

async function loadGlobalRef() {
  try {
    const data = await invoke<{ page_size: number; is_custom: boolean }>(
      'get_page_size_detail',
      { scope: null },
    )
    globalRef.value = { page_size: data.page_size }
  } catch {
    globalRef.value = null
  }
}

function toggleGlobalRef() {
  showGlobalRef.value = !showGlobalRef.value
  if (showGlobalRef.value && !globalRef.value) loadGlobalRef()
}

onMounted(async () => {
  try {
    const data = await invoke<{ schemas: Array<{ schema: string }>; current_schema: string }>(
      'get_schemas',
    )
    schemas.value = data.schemas.map((s) => s.schema)
    selectedSchema.value = data.current_schema || schemas.value[0] || ''
  } catch (e) {
    // 方案列表加载失败时静默，界面仍可用（候选词数面板会自行报错）
  }
})
</script>

<template>
  <div class="scheme-profile">
    <DeployNotice />

    <div class="profile-header">
      <label class="profile-label">方案:</label>
      <select v-model="selectedSchema" class="schema-select">
        <option v-for="s in schemas" :key="s" :value="s">{{ s }}</option>
      </select>
      <span v-if="schemas.length === 0" class="hint">未加载到方案列表</span>
    </div>

    <!-- 全局默认配置参考区（只读说明） -->
    <div class="default-ref">
      <p>🌐 全局默认配置为各方案的基础，方案自定义的配置项以方案为准，<strong>优先生效</strong>。</p>
      <p class="sub">
        下方配置写入 <code>{{ selectedSchema }}.custom.yaml</code>；未自定义的项显示「默认」，修改后保存即转为自定义。
      </p>
      <button class="ref-toggle" @click="toggleGlobalRef">
        {{ showGlobalRef ? '收起' : '查看' }}全局默认配置
      </button>
      <div v-if="showGlobalRef" class="global-ref-body">
        <p class="sub">以下为全局 <code>default.custom.yaml</code> 的当前默认值（只读参考）：</p>
        <ul class="ref-list">
          <li>候选词数：<code>{{ globalRef?.page_size ?? '-' }}</code></li>
          <li>标点符号：在「标点符号」页查看</li>
          <li>快捷键：在「快捷键」页查看</li>
        </ul>
      </div>
    </div>

    <!-- 内部 Tab -->
    <div class="profile-tabs">
      <button
        :class="['profile-tab', { active: activeTab === 'punctuation' }]"
        @click="activeTab = 'punctuation'"
      >
        标点符号
      </button>
      <button
        :class="['profile-tab', { active: activeTab === 'keybinding' }]"
        @click="activeTab = 'keybinding'"
      >
        快捷键
      </button>
      <button
        :class="['profile-tab', { active: activeTab === 'candidate' }]"
        @click="activeTab = 'candidate'"
      >
        候选词数
      </button>
    </div>

    <div class="profile-body">
      <PageSizeSettings v-if="activeTab === 'candidate'" :scope="selectedSchema" />
      <!-- Task 7/8 接入：标点、快捷键 -->
      <PunctuationSettings v-else-if="activeTab === 'punctuation'" :scope="selectedSchema" />
      <KeybindingEditor v-else-if="activeTab === 'keybinding'" :scope="selectedSchema" />
    </div>
  </div>
</template>

<style scoped>
.scheme-profile {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 700px;
}

.profile-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.profile-label {
  font-size: 14px;
  color: var(--color-text-secondary);
}

.schema-select {
  padding: 6px 10px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 14px;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
  min-width: 200px;
}

.default-ref {
  padding: 12px 16px;
  background: var(--color-accent-muted);
  border: 1px solid var(--color-accent-light);
  border-radius: 8px;
  font-size: 13px;
  line-height: 1.6;
}

.default-ref .sub {
  margin-top: 4px;
  color: var(--color-text-secondary);
  font-size: 12px;
}

.ref-toggle {
  margin-top: 8px;
  padding: 4px 12px;
  border: 1px solid var(--color-accent);
  background: transparent;
  color: var(--color-accent);
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
}

.ref-toggle:hover {
  background: var(--color-accent-muted);
}

.global-ref-body {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px dashed var(--color-border);
}

.ref-list {
  margin-top: 6px;
  padding-left: 18px;
  font-size: 13px;
  line-height: 1.8;
}

.default-ref code {
  background: var(--color-bg-active);
  padding: 1px 5px;
  border-radius: 4px;
  font-size: 12px;
}

.profile-tabs {
  display: flex;
  gap: 4px;
  background: var(--color-bg-active);
  padding: 4px;
  border-radius: 8px;
  width: fit-content;
}

.profile-tab {
  padding: 6px 16px;
  border: none;
  background: none;
  color: var(--color-text-primary);
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.profile-tab.active {
  background: var(--color-bg-secondary);
  box-shadow: var(--shadow-sm);
}

.profile-body {
  margin-top: 4px;
}
</style>
