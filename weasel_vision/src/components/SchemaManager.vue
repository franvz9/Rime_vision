<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import DeployNotice from './DeployNotice.vue'


interface Schema {
  schema: string
  enabled: boolean
}

const schemas = ref<Schema[]>([])
const currentSchema = ref<string>('')
const pendingDeleteSchemas = ref<Set<string>>(new Set())

onMounted(async () => {
  try {
    const data: any = await invoke('get_schemas')
    schemas.value = data.schemas
    currentSchema.value = data.current_schema || ''
  } catch (e) {
    console.error('Failed to load schemas:', e)
  }
})

function toggleSchema(schema: Schema) {
  schema.enabled = !schema.enabled
}

async function save() {
  try {
    await invoke('save_schemas', { schemas: schemas.value })
  } catch (e) {
    console.error('Failed to save schemas:', e)
  }
}

async function importSchema() {
  const selected = await dialogOpen({
    directory: false,
    multiple: false,
    title: '选择输入方案文件',
    filters: [{ name: 'YAML Files', extensions: ['yaml', 'yml'] }]
  })
  if (selected && typeof selected === 'string') {
    try {
      await invoke('import_schema', { filePath: selected })
      const data: any = await invoke('get_schemas')
      schemas.value = data.schemas
      currentSchema.value = data.current_schema || ''
      alert('输入方案导入成功！')
    } catch (e) {
      console.error('Failed to import schema:', e)
      alert('导入失败：' + String(e))
    }
  }
}

function canDeleteSchema(schemaId: string): boolean {
  // Cannot delete the currently active schema
  if (currentSchema.value === schemaId) return false
  return true
}

function toggleDeleteSchema(schemaId: string) {
  if (!canDeleteSchema(schemaId)) return
  const filename = `${schemaId}.schema.yaml`
  if (pendingDeleteSchemas.value.has(schemaId)) {
    pendingDeleteSchemas.value.delete(schemaId)
    window.dispatchEvent(new CustomEvent('remove-pending-delete', {
      detail: { delete_type: 'schema', identifier: filename }
    }))
  } else {
    pendingDeleteSchemas.value.add(schemaId)
    window.dispatchEvent(new CustomEvent('add-pending-delete', {
      detail: { delete_type: 'schema', identifier: filename, label: `方案: ${schemaId}` }
    }))
  }
  pendingDeleteSchemas.value = new Set(pendingDeleteSchemas.value)
}
</script>

<template>
  <div class="schema-manager">
    <DeployNotice />

    <div class="section">
      <div class="section-header">
        <h3>输入方案列表</h3>
        <button class="btn btn-sm" @click="importSchema">📥 导入方案</button>
      </div>
      <p class="hint">启用的方案将显示在输入法方案切换菜单中</p>

      <div class="schema-list">
        <div
          v-for="schema in schemas"
          :key="schema.schema"
          :class="['schema-item', { active: currentSchema === schema.schema, 'is-deleting': pendingDeleteSchemas.has(schema.schema) }]"
        >
          <label class="toggle">
            <input type="checkbox" :checked="schema.enabled" @change="toggleSchema(schema)" />
            <span class="toggle-slider"></span>
          </label>
          <span class="schema-id">{{ schema.schema }}</span>
          <span v-if="currentSchema === schema.schema && !pendingDeleteSchemas.has(schema.schema)" class="badge badge-active">使用中</span>
          <span v-if="pendingDeleteSchemas.has(schema.schema)" class="badge badge-deleting">待删除</span>
          <button
            v-if="pendingDeleteSchemas.has(schema.schema)"
            class="btn-cancel-delete"
            @click.stop="toggleDeleteSchema(schema.schema)"
          >取消删除</button>
          <button
            v-else
            class="btn-delete"
            :disabled="!canDeleteSchema(schema.schema)"
            :title="!canDeleteSchema(schema.schema) ? '使用中不可删除' : '删除'"
            @click.stop="toggleDeleteSchema(schema.schema)"
          >🗑</button>
        </div>
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

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.section-header h3 {
  margin: 0;
}

.btn-sm {
  padding: 4px 12px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
}

.hint {
  font-size: 13px;
  color: var(--color-text-secondary);
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
  background: var(--color-bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--color-border);
}

.schema-item.active {
  border-color: var(--color-success);
  background: var(--color-success-muted);
}

.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
}

.badge-active {
  background: var(--color-success);
  color: white;
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
  background: var(--color-border-dark);
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
  background: var(--color-bg-secondary);
  border-radius: 50%;
  transition: 0.3s;
}

.toggle input:checked + .toggle-slider {
  background: var(--color-accent);
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
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 14px;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
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
  color: var(--color-text-primary);
}

.btn-primary {
  background: var(--color-accent);
  color: white;
}

.schema-item.is-deleting {
  opacity: 0.5;
  background: var(--color-bg-tertiary);
  border: 1px dashed var(--color-border-dark);
}

.badge-deleting {
  background: var(--color-danger);
  color: white;
}

.btn-delete {
  padding: 4px 8px;
  border: 1px solid var(--color-danger);
  background: var(--color-bg-secondary);
  color: var(--color-danger);
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  margin-left: auto;
}

.btn-delete:hover:not(:disabled) {
  background: var(--color-danger);
  color: white;
}

.btn-delete:disabled {
  color: var(--color-text-tertiary);
  border-color: var(--color-border);
  cursor: not-allowed;
  opacity: 0.5;
}

.btn-cancel-delete {
  padding: 4px 8px;
  border: 1px solid var(--color-success);
  background: var(--color-bg-secondary);
  border-radius: 4px;
  font-size: 11px;
  cursor: pointer;
  color: var(--color-success);
  margin-left: auto;
}

.btn-cancel-delete:hover {
  background: var(--color-success);
  color: white;
}

</style>
