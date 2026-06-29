<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import DeployNotice from './DeployNotice.vue'


interface KeyBinding {
  when: string
  accept: string
  send: string
  toggle: string
  select: string
}

// 预定义的选项列表（后端参数 -> 显示文本）
const whenOptions = [
  { value: 'composing', label: 'composing (正在输入中)' },
  { value: 'always', label: 'always (始终)' },
  { value: 'has_menu', label: 'has_menu (有候选词时)' },
  { value: 'paging', label: 'paging (翻页时)' },
]

const sendOptions = [
  { value: '', label: '(空)' },
  { value: 'Up', label: 'Up (上箭头)' },
  { value: 'Down', label: 'Down (下箭头)' },
  { value: 'Left', label: 'Left (左箭头)' },
  { value: 'Right', label: 'Right (右箭头)' },
  { value: 'Home', label: 'Home (行首)' },
  { value: 'End', label: 'End (行尾)' },
  { value: 'Page_Up', label: 'Page_Up (上一页)' },
  { value: 'Page_Down', label: 'Page_Down (下一页)' },
  { value: 'Return', label: 'Return (回车)' },
  { value: 'Escape', label: 'Escape (ESC)' },
  { value: 'BackSpace', label: 'BackSpace (退格)' },
  { value: 'Delete', label: 'Delete (删除)' },
  { value: 'Tab', label: 'Tab (制表符)' },
  { value: 'Space', label: 'Space (空格)' },
]

const toggleOptions = [
  { value: '', label: '(空)' },
  { value: 'ascii_mode', label: 'ascii_mode (中英文切换)' },
  { value: 'full_shape', label: 'full_shape (全角/半角)' },
  { value: 'simplification', label: 'simplification (简繁切换)' },
  { value: 'extended_charset', label: 'extended_charset (扩展字符集)' },
  { value: 'ascii_punct', label: 'ascii_punct (标点符号切换)' },
]

const selectOptions = [
  { value: '', label: '(空)' },
  { value: '1', label: '1 (第一个候选词)' },
  { value: '2', label: '2 (第二个候选词)' },
  { value: '3', label: '3 (第三个候选词)' },
  { value: '4', label: '4 (第四个候选词)' },
  { value: '5', label: '5 (第五个候选词)' },
  { value: '6', label: '6 (第六个候选词)' },
  { value: '7', label: '7 (第七个候选词)' },
  { value: '8', label: '8 (第八个候选词)' },
  { value: '9', label: '9 (第九个候选词)' },
  { value: '0', label: '0 (第十个候选词)' },
]

const bindings = ref<KeyBinding[]>([])

onMounted(async () => {
  try {
    bindings.value = await invoke('get_keybindings')
  } catch (e) {
    console.error('Failed to load keybindings:', e)
  }
})

function addBinding() {
  bindings.value.push({
    when: 'composing',
    accept: '',
    send: '',
    toggle: '',
    select: '',
  })
}

function removeBinding(index: number) {
  bindings.value.splice(index, 1)
}

async function save() {
  try {
    await invoke('save_keybindings', { bindings: bindings.value })
  } catch (e) {
    console.error('Failed to save keybindings:', e)
  }
}
</script>

<template>
  <div class="keybinding-editor">
    <DeployNotice />

    <div class="section">
      <h3>快捷键绑定</h3>
      <p class="hint">配置全局快捷键绑定规则，定义在特定条件下按某个键时执行的操作</p>
      
      <div class="info-box">
        <strong>💡 使用说明：</strong>
        <ul>
          <li><strong>条件 (when)</strong>：触发条件，如 <code>composing</code>（正在输入中）、<code>always</code>（始终）等</li>
          <li><strong>按键 (accept)</strong>：要捕获的按键组合，如 <code>Control+p</code>、<code>F4</code> 等</li>
          <li><strong>发送 (send)</strong>：向应用程序发送的按键，如 <code>Up</code>、<code>Page_Up</code> 等</li>
          <li><strong>切换 (toggle)</strong>：切换的状态，如 <code>ascii_mode</code>（中英文切换）、<code>simplification</code>（简繁切换）等</li>
          <li><strong>选择 (select)</strong>：选择候选词的位置，如 <code>1</code>（第一个）、<code>2</code>（第二个）等</li>
        </ul>
        <p><strong>示例：</strong><code>Control+n</code> → <code>Down</code> 表示在输入过程中按 Ctrl+N 向下移动光标</p>
      </div>

      <div class="binding-list">
        <div v-for="(binding, index) in bindings" :key="index" class="binding-item">
          <div class="binding-fields">
            <div class="field">
              <label>条件 (when)</label>
              <select v-model="binding.when">
                <option v-for="opt in whenOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>
            <div class="field">
              <label>按键 (accept)</label>
              <input v-model="binding.accept" placeholder="如 Control+p" />
            </div>
            <div class="field">
              <label>发送 (send)</label>
              <select v-model="binding.send">
                <option v-for="opt in sendOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>
            <div class="field">
              <label>切换 (toggle)</label>
              <select v-model="binding.toggle">
                <option v-for="opt in toggleOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>
            <div class="field">
              <label>选择 (select)</label>
              <select v-model="binding.select">
                <option v-for="opt in selectOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>
          </div>
          <button class="remove-btn" @click="removeBinding(index)">×</button>
        </div>
      </div>

      <button class="add-btn" @click="addBinding">+ 添加绑定</button>
    </div>

    <div class="actions">
      <button class="btn btn-primary" @click="save">保存</button>
    </div>
  </div>
</template>

<style scoped>
.keybinding-editor {
  max-width: 700px;
}

.section h3 {
  font-size: 15px;
  margin-bottom: 4px;
}

.hint {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.info-box {
  background: var(--color-accent-muted);
  border-left: 4px solid var(--color-accent);
  padding: 12px 16px;
  margin-bottom: 16px;
  border-radius: 4px;
}

.info-box strong {
  color: var(--color-accent-hover);
}

.info-box ul {
  margin: 8px 0 0 20px;
  padding: 0;
}

.info-box li {
  font-size: 13px;
  line-height: 1.6;
  margin-bottom: 4px;
}

.info-box code {
  background: var(--color-accent-muted);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: monospace;
  font-size: 12px;
}

.info-box p {
  margin-top: 8px;
  font-size: 13px;
}

.binding-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.binding-item {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  padding: 12px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
}

.binding-fields {
  flex: 1;
  display: grid;
  grid-template-columns: 1.2fr 1.5fr 1fr 1.3fr 1fr;
  gap: 8px;
}

.field label {
  display: block;
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-bottom: 4px;
}

.field input,
.field select {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  font-size: 13px;
  font-family: monospace;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
  cursor: pointer;
}

.field select:hover {
  border-color: var(--color-accent);
}

.remove-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: var(--color-btn-danger-bg);
  color: var(--color-text-inverse);
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
}

.remove-btn:hover {
  background: var(--color-btn-danger-hover);
}

.add-btn {
  margin-top: 12px;
  padding: 8px 16px;
  border: 1px dashed var(--color-border-dark);
  background: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--color-accent);
}

.add-btn:hover {
  background: var(--color-accent-muted);
}

.actions {
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
  background: var(--color-btn-primary-bg);
  color: var(--color-text-inverse);
}

.btn-primary:hover {
  background: var(--color-btn-primary-hover);
}
</style>
