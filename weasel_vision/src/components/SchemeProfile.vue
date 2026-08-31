<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import DeployNotice from './DeployNotice.vue'
import PageSizeSettings from './PageSizeSettings.vue'
import PunctuationSettings from './PunctuationSettings.vue'
import KeybindingEditor from './KeybindingEditor.vue'
import TranslatorSettings from './TranslatorSettings.vue'

/** 哨兵值：空字符串代表「全局默认」范围 */
const GLOBAL_SCOPE = ''

const schemas = ref<string[]>([])
const selectedScope = ref<string>(GLOBAL_SCOPE)
const activeTab = ref<'punctuation' | 'keybinding' | 'candidate' | 'translator'>('punctuation')

/** 子面板 ref，用于统一在顶部保存 */
const punctRef = ref<InstanceType<typeof PunctuationSettings> | null>(null)
const kbRef = ref<InstanceType<typeof KeybindingEditor> | null>(null)
const pageSizeRef = ref<InstanceType<typeof PageSizeSettings> | null>(null)
const translatorRef = ref<InstanceType<typeof TranslatorSettings> | null>(null)

/** 是否当前查看全局默认配置 */
const isGlobal = computed(() => selectedScope.value === GLOBAL_SCOPE)

/** 当前范围对应的自定义文件名（用于文案提示） */
const scopeFile = computed(() =>
  isGlobal.value ? 'default.custom.yaml' : `${selectedScope.value}.custom.yaml`,
)

/** 保存当前激活子面板的配置（调用子面板暴露的 save 方法） */
function saveActive() {
  if (activeTab.value === 'punctuation') punctRef.value?.save()
  else if (activeTab.value === 'keybinding') kbRef.value?.save()
  else if (activeTab.value === 'candidate') pageSizeRef.value?.save()
  else translatorRef.value?.save()
}

onMounted(async () => {
  try {
    const data = await invoke<{ schemas: Array<{ schema: string }>; current_schema: string }>(
      'get_schemas',
    )
    schemas.value = data.schemas.map((s) => s.schema)
  } catch {
    // 方案列表加载失败时静默，界面仍可用（候选词数面板会自行报错）
  }
})
</script>

<template>
  <div class="scheme-profile">
    <div class="scheme-scroll">
      <DeployNotice />

      <div class="profile-header">
        <label class="profile-label">配置范围:</label>
        <select v-model="selectedScope" class="schema-select">
          <option :value="GLOBAL_SCOPE">全局默认</option>
          <option v-for="s in schemas" :key="s" :value="s">{{ s }}</option>
        </select>
        <span v-if="schemas.length === 0" class="hint">未加载到方案列表</span>
      </div>

      <!-- 范围说明区 -->
      <div class="default-ref">
        <template v-if="isGlobal">
          <p>🌐 正在查看<strong>全局默认配置</strong>，它作为所有方案的基础。</p>
          <p class="sub">
            下方配置写入 <code>{{ scopeFile }}</code
            >；未自定义的项显示「默认」，修改后保存即转为自定义。
          </p>
        </template>
        <template v-else>
          <p>
            🌐 全局默认配置为各方案的基础，方案自定义的配置项以方案为准，<strong>优先生效</strong>。
          </p>
          <p class="sub">
            下方配置写入
            <code>{{ scopeFile }}</code
            >；未自定义的项显示「默认」，修改后保存即转为自定义。
          </p>
        </template>
      </div>

      <!-- 内部 Tab -->
      <div class="profile-tabs">
        <div class="profile-tab-group">
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
            :class="['profile-tab', { active: activeTab === 'translator' }]"
            @click="activeTab = 'translator'"
          >
            翻译器
          </button>
          <button
            :class="['profile-tab', { active: activeTab === 'candidate' }]"
            @click="activeTab = 'candidate'"
          >
            候选词数
          </button>
        </div>
        <button class="wv-btn wv-btn-save" @click="saveActive">保存当前配置</button>
      </div>

      <div class="profile-body">
        <!-- 用 v-show 而非 v-if：保持子面板常驻挂载，切 Tab 不丢失未保存编辑 -->
        <PageSizeSettings
          v-show="activeTab === 'candidate'"
          ref="pageSizeRef"
          :scope="selectedScope"
        />
        <PunctuationSettings
          v-show="activeTab === 'punctuation'"
          ref="punctRef"
          :scope="selectedScope"
        />
        <KeybindingEditor v-show="activeTab === 'keybinding'" ref="kbRef" :scope="selectedScope" />
        <TranslatorSettings
          v-show="activeTab === 'translator'"
          ref="translatorRef"
          :scope="selectedScope"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.scheme-profile {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.scheme-scroll {
  flex: 1 1 auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
  padding-right: 4px;
}

.scheme-scroll > * {
  max-width: 720px;
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

.default-ref code {
  background: var(--color-bg-active);
  padding: 1px 5px;
  border-radius: 4px;
  font-size: 12px;
}

.profile-tabs {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.profile-tab-group {
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

.wv-btn-save {
  padding: 8px 20px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  background: var(--color-btn-primary-bg);
  color: var(--color-text-inverse);
}

.wv-btn-save:hover {
  background: var(--color-btn-primary-hover);
}

.profile-body {
  margin-top: 4px;
}
</style>
