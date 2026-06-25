<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['changed'])

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

onMounted(async () => {
  try {
    settings.value = await invoke('get_general_settings')
  } catch (e) {
    console.error('Failed to load settings:', e)
  }
})

const capsLockOptions = ['commit_code', 'inline_ascii', 'noop', 'clear']
const shiftOptions = ['commit_code', 'inline_ascii', 'noop']

async function save() {
  try {
    await invoke('save_general_settings', { settings: settings.value })
    showSaved.value = true
    emit('changed')
    setTimeout(() => { showSaved.value = false }, 2000)
  } catch (e) {
    console.error('Failed to save settings:', e)
  }
}
</script>

<template>
  <div class="general-settings">
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
        <input v-model="settings.switcher_hotkeys" />
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
          <option v-for="opt in capsLockOptions" :key="opt" :value="opt">{{ opt }}</option>
        </select>
      </div>
      <div class="form-row">
        <label>左 Shift:</label>
        <select v-model="settings.shift_left_action">
          <option v-for="opt in shiftOptions" :key="opt" :value="opt">{{ opt }}</option>
        </select>
      </div>
      <div class="form-row">
        <label>右 Shift:</label>
        <select v-model="settings.shift_right_action">
          <option v-for="opt in shiftOptions" :key="opt" :value="opt">{{ opt }}</option>
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
  border-bottom: 1px solid #e5e5e5;
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
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
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
}

.btn-primary {
  background: #007aff;
  color: white;
}

.saved-hint {
  color: #34c759;
  font-size: 14px;
}
</style>
