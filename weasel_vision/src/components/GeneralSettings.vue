<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import DeployNotice from './DeployNotice.vue'
import { useToast } from '../composables/useToast'
import { errorMessage } from '../utils'
import type { GeneralSettings } from '../../src-tauri/bindings/GeneralSettings'

const toast = useToast()

const settings = ref<GeneralSettings>({
  switcher_caption: '〔方案切换〕',
  switcher_hotkeys: 'Control+grave,Control+Shift+grave',
  switcher_fold_options: true,
  switcher_abbreviate_options: true,
  caps_lock_action: 'commit_code',
  shift_left_action: 'commit_code',
  shift_right_action: 'inline_ascii',
  good_old_caps_lock: true,
})

const showSaved = ref(false)

// 按键名称映射（用于显示）
const keyDisplayMap: Record<string, string> = {
  grave: '` (反引号)',
  minus: '- (减号)',
  equal: '= (等号)',
  bracketleft: '[ (左方括号)',
  bracketright: '] (右方括号)',
  backslash: '\\ (反斜杠)',
  semicolon: '; (分号)',
  apostrophe: "' (单引号)",
  comma: ', (逗号)',
  period: '. (句号)',
  slash: '/ (斜杠)',
}

// 将按键名转换为显示文本
function formatKeyDisplay(keyName: string): string {
  if (!keyName) return ''
  const keys = keyName.split(/[,\s]+/).filter((k) => k.trim())
  return keys
    .map((key) => {
      // 处理组合键，如 Control+grave
      const parts = key.split('+')
      return parts
        .map((part) => {
          const lowerPart = part.toLowerCase()
          return keyDisplayMap[lowerPart] || part
        })
        .join('+')
    })
    .join(', ')
}

let mounted = true
let saveToastTimer: ReturnType<typeof setTimeout> | null = null

onMounted(async () => {
  try {
    const data = await invoke<GeneralSettings>('get_general_settings')
    if (!mounted) return
    settings.value = data
  } catch (e) {
    if (!mounted) return
    toast.error(`加载通用设置失败: ${errorMessage(e)}`)
  }
})

onUnmounted(() => {
  mounted = false
  if (saveToastTimer) clearTimeout(saveToastTimer)
})

// Caps Lock 动作选项（后端参数 -> 显示文本）
const capsLockOptions = [
  { value: 'commit_code', label: 'commit_code (上屏原始编码)' },
  { value: 'inline_ascii', label: 'inline_ascii (临时英文模式)' },
  { value: 'noop', label: 'noop (无操作)' },
  { value: 'clear', label: 'clear (清空输入)' },
]

// Shift 动作选项（后端参数 -> 显示文本）
const shiftOptions = [
  { value: 'commit_code', label: 'commit_code (上屏原始编码)' },
  { value: 'inline_ascii', label: 'inline_ascii (临时英文模式)' },
  { value: 'noop', label: 'noop (无操作)' },
]

async function save() {
  try {
    await invoke('save_general_settings', { settings: settings.value })
    showSaved.value = true
    saveToastTimer = setTimeout(() => {
      showSaved.value = false
    }, 2000)
  } catch (e) {
    toast.error(`保存设置失败: ${errorMessage(e)}`)
  }
}
</script>

<template>
  <div class="general-settings">
    <div class="settings-scroll">
      <DeployNotice />

      <div class="section">
        <div class="section-header">
          <h3>方案切换器 (switcher)</h3>
          <div class="section-header-actions">
            <span v-if="showSaved" class="wv-saved-hint">已保存</span>
            <button class="wv-btn wv-btn-primary" @click="save">保存</button>
          </div>
        </div>
        <div class="form-row">
          <label>切换标题:</label>
          <input v-model="settings.switcher_caption" />
        </div>
        <div class="form-row">
          <label>快捷键:</label>
          <div class="hotkey-display">
            <input v-model="settings.switcher_hotkeys" class="hotkey-input" />
            <span class="hotkey-hint">{{ formatKeyDisplay(settings.switcher_hotkeys) }}</span>
          </div>
        </div>
        <label class="checkbox">
          <input v-model="settings.switcher_fold_options" type="checkbox" />
          折叠选项 (fold_options)
        </label>
        <label class="checkbox">
          <input v-model="settings.switcher_abbreviate_options" type="checkbox" />
          缩写选项 (abbreviate_options)
        </label>
      </div>

      <div class="section">
        <h3>中英文切换 (ascii_composer)</h3>
        <label class="checkbox">
          <input v-model="settings.good_old_caps_lock" type="checkbox" />
          经典 Caps Lock 模式
        </label>
        <div class="form-row">
          <label>Caps Lock:</label>
          <select v-model="settings.caps_lock_action">
            <option v-for="opt in capsLockOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>
        <div class="form-row">
          <label>左 Shift:</label>
          <select v-model="settings.shift_left_action">
            <option v-for="opt in shiftOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>
        <div class="form-row">
          <label>右 Shift:</label>
          <select v-model="settings.shift_right_action">
            <option v-for="opt in shiftOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.general-settings {
  height: 100%;
  max-width: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.settings-scroll {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  padding-right: 4px;
}

.settings-scroll > * {
  max-width: 720px;
}

.section {
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--color-border);
}

.section h3 {
  font-size: 15px;
  margin-bottom: 12px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-header h3 {
  margin: 0;
}

.section-header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.wv-saved-hint {
  font-size: 13px;
  color: var(--color-success);
}

.form-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.form-row label {
  min-width: 100px;
  font-size: 14px;
}

.form-row input,
.form-row select {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 14px;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
}

.hotkey-display {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.hotkey-input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 14px;
  font-family: monospace;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
}

.hotkey-hint {
  font-size: 12px;
  color: var(--color-text-secondary);
  background: var(--color-bg-tertiary);
  padding: 4px 8px;
  border-radius: 4px;
}

.checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  font-size: 14px;
  cursor: pointer;
}

.wv-btn {
  padding: 8px 20px;
  border: none;
  font-size: 14px;
}
</style>
