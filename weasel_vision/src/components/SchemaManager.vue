<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['changed'])

interface Schema {
  schema: string
  enabled: boolean
}

const schemas = ref<Schema[]>([])
const pageSize = ref(6)

onMounted(async () => {
  try {
    const data: any = await invoke('get_schemas')
    schemas.value = data.schemas
    pageSize.value = data.page_size
  } catch (e) {
    console.error('Failed to load schemas:', e)
  }
})

function toggleSchema(schema: Schema) {
  schema.enabled = !schema.enabled
  emit('changed')
}

async function save() {
  try {
    await invoke('save_schemas', { schemas: schemas.value, pageSize: pageSize.value })
    emit('changed')
  } catch (e) {
    console.error('Failed to save schemas:', e)
  }
}
</script>

<template>
  <div class="schema-manager">
    <div class="section">
      <h3>输入方案列表</h3>
      <p class="hint">启用的方案将显示在输入法方案切换菜单中</p>

      <div class="schema-list">
        <div
          v-for="schema in schemas"
          :key="schema.schema"
          class="schema-item"
        >
          <label class="toggle">
            <input type="checkbox" :checked="schema.enabled" @change="toggleSchema(schema)" />
            <span class="toggle-slider"></span>
          </label>
          <span class="schema-id">{{ schema.schema }}</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h3>候选词设置</h3>
      <div class="form-row">
        <label>每页候选词数:</label>
        <input type="number" v-model.number="pageSize" min="3" max="10" />
      </div>
    </div>

    <div class="actions">
      <button class="btn btn-primary" @click="save">保存</button>
    </div>
  </div>
</template>

<style scoped>
.schema-manager {
  max-width: 600px;
}

.section {
  margin-bottom: 24px;
}

.section h3 {
  font-size: 15px;
  margin-bottom: 8px;
}

.hint {
  font-size: 13px;
  color: #666;
  margin-bottom: 12px;
}

.schema-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.schema-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: white;
  border-radius: 8px;
  border: 1px solid #e5e5e5;
}

.schema-id {
  font-size: 14px;
  font-family: monospace;
}

.toggle {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background: #ccc;
  border-radius: 22px;
  transition: 0.3s;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  height: 18px;
  width: 18px;
  left: 2px;
  bottom: 2px;
  background: white;
  border-radius: 50%;
  transition: 0.3s;
}

.toggle input:checked + .toggle-slider {
  background: #007aff;
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(18px);
}

.form-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 8px;
}

.form-row label {
  font-size: 14px;
}

.form-row input {
  width: 80px;
  padding: 6px 8px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
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
