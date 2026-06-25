<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['changed'])

interface KeyBinding {
  when: string
  accept: string
  send: string
  toggle: string
  select: string
}

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
  emit('changed')
}

function removeBinding(index: number) {
  bindings.value.splice(index, 1)
  emit('changed')
}

async function save() {
  try {
    await invoke('save_keybindings', { bindings: bindings.value })
    emit('changed')
  } catch (e) {
    console.error('Failed to save keybindings:', e)
  }
}
</script>

<template>
  <div class="keybinding-editor">
    <div class="section">
      <h3>快捷键绑定</h3>
      <p class="hint">配置全局快捷键绑定规则</p>

      <div class="binding-list">
        <div v-for="(binding, index) in bindings" :key="index" class="binding-item">
          <div class="binding-fields">
            <div class="field">
              <label>条件 (when)</label>
              <input v-model="binding.when" placeholder="composing" />
            </div>
            <div class="field">
              <label>按键 (accept)</label>
              <input v-model="binding.accept" placeholder="Control+p" />
            </div>
            <div class="field">
              <label>发送 (send)</label>
              <input v-model="binding.send" placeholder="Page_Up" />
            </div>
            <div class="field">
              <label>切换 (toggle)</label>
              <input v-model="binding.toggle" placeholder="ascii_mode" />
            </div>
            <div class="field">
              <label>选择 (select)</label>
              <input v-model="binding.select" placeholder="1" />
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
  color: #666;
  margin-bottom: 16px;
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
  background: white;
  border: 1px solid #e5e5e5;
  border-radius: 8px;
}

.binding-fields {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 8px;
}

.field label {
  display: block;
  font-size: 11px;
  color: #666;
  margin-bottom: 4px;
}

.field input {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 13px;
  font-family: monospace;
}

.remove-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: #ff3b30;
  color: white;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
}

.add-btn {
  margin-top: 12px;
  padding: 8px 16px;
  border: 1px dashed #ccc;
  background: none;
  border-radius: 6px;
  cursor: pointer;
  color: #007aff;
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
}

.btn-primary {
  background: #007aff;
  color: white;
}
</style>
