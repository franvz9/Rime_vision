<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import DeployNotice from './DeployNotice.vue'
import { useToast } from '../composables/useToast'
import { errorMessage } from '../utils'

const toast = useToast()


interface GeneralSettings {
  page_size: number
  enable_encoder: boolean
  enable_sentence: boolean
  enable_user_dict: boolean
  encode_commit_history: boolean
  switcher_caption: string
  switcher_hotkeys: string
  switcher_fold_options: boolean
  switcher_abbreviate_options: boolean
  caps_lock_action: string
  shift_left_action: string
  shift_right_action: string
  good_old_caps_lock: boolean
}

const settings = ref<GeneralSettings>({
  page_size: 6,
  enable_encoder: true,
  enable_sentence: true,
  enable_user_dict: true,
  encode_commit_history: true,
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
  'grave': '` (反引号)',
  'minus': '- (减号)',
  'equal': '= (等号)',
  'bracketleft': '[ (左方括号)',
  'bracketright': '] (右方括号)',
  'backslash': '\\ (反斜杠)',
  'semicolon': '; (分号)',
  'apostrophe': "' (单引号)",
  'comma': ', (逗号)',
  'period': '. (句号)',
  'slash': '/ (斜杠)',
}

// 将按键名转换为显示文本
function formatKeyDisplay(keyName: string): string {
  if (!keyName) return ''
  const keys = keyName.split(/[,\s]+/).filter(k => k.trim())
  return keys.map(key => {
    // 处理组合键，如 Control+grave
    const parts = key.split('+')
    return parts.map(part => {
      const lowerPart = part.toLowerCase()
      return keyDisplayMap[lowerPart] || part
    }).join('+')
  }).join(', ')
}

let mounted = true

onMounted(async () => {
  try {
    settings.value = await invoke('get_general_settings')
    if (!mounted) return
  } catch (e) {
    if (!mounted) return
    toast.error(`加载通用设置失败: ${errorMessage(e)}`)
  }
})

onUnmounted(() => { mounted = false })

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
    setTimeout(() => { showSaved.value = false }, 2000)
  } catch (e) {
    toast.error(`保存设置失败: ${errorMessage(e)}`)
  }
}
</script>

<template>
  <div class="general-settings">
    <DeployNotice />

    <div class="section">
      <h3>候选词</h3>
      <div class="form-row">
        <label>每页候选词数:</label>
        <input type="number" v-model.number="settings.page_size" min="3" max="10" />
      </div>
    </div>

    <div class="section">
      <h3>翻译器</h3>
      <label class="checkbox">
        <input type="checkbox" v-model="settings.enable_encoder" />
        启用自动造词 (enable_encoder)
      </label>
      <label class="checkbox">
        <input type="checkbox" v-model="settings.enable_sentence" />
        启用自动句子输入 (enable_sentence)
      </label>
      <label class="checkbox">
        <input type="checkbox" v-model="settings.enable_user_dict" />
        启用用户词典 (enable_user_dict)
      </label>
      <label class="checkbox">
        <input type="checkbox" v-model="settings.encode_commit_history" />
        自动编码上屏词语 (encode_commit_history)
      </label>
    </div>

    <div class="section">
      <h3>方案切换器 (switcher)</h3>
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
        <input type="checkbox" v-model="settings.switcher_fold_options" />
        折叠选项 (fold_options)
      </label>
      <label class="checkbox">
        <input type="checkbox" v-model="settings.switcher_abbreviate_options" />
        缩写选项 (abbreviate_options)
      </label>
    </div>

    <div class="section">
      <h3>中英文切换 (ascii_composer)</h3>
      <label class="checkbox">
        <input type="checkbox" v-model="settings.good_old_caps_lock" />
        经典 Caps Lock 模式
      </label>
      <div class="form-row">
        <label>Caps Lock:</label>
        <select v-model="settings.caps_lock_action">
          <option v-for="opt in capsLockOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
        </select>
      </div>
      <div class="form-row">
        <label>左 Shift:</label>
        <select v-model="settings.shift_left_action">
          <option v-for="opt in shiftOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
        </select>
      </div>
      <div class="form-row">
        <label>右 Shift:</label>
        <select v-model="settings.shift_right_action">
          <option v-for="opt in shiftOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
        </select>
      </div>
    </div>

    <div class="actions">
      <span v-if="showSaved" class="saved-hint">已保存</span>
      <button class="btn btn-primary" @click="save">保存</button>
    </div>
  </div>
</template>

<style scoped>
.general-settings {
  max-width: 500px;
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

.actions {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
}

.btn {
  padding: 8px 20px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
}

.btn-primary {
  background: var(--color-accent);
  color: white;
}

.saved-hint {
  color: var(--color-success);
  font-size: 14px;
}
</style>
