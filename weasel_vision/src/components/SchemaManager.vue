<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import DeployNotice from './DeployNotice.vue'
import { useToast } from '../composables/useToast'
import { errorMessage, emitBusEvent, BusEvents } from '../utils'
import type { SchemaListData } from '../../src-tauri/bindings/SchemaListData'
import type { RimeSchema } from '../../src-tauri/bindings/RimeSchema'

const toast = useToast()

const schemas = ref<RimeSchema[]>([])
const currentSchema = ref<string>('')
const pendingDeleteSchemas = ref<Set<string>>(new Set())
/** 右侧预览的方案（默认跟随当前使用中方案） */
const previewSchema = ref<RimeSchema | null>(null)

let schemaMounted = true

onMounted(async () => {
  try {
    const data: SchemaListData = await invoke('get_schemas')
    if (!schemaMounted) return
    schemas.value = data.schemas
    currentSchema.value = data.current_schema || ''
    previewSchema.value =
      schemas.value.find((s) => s.schema === currentSchema.value) || schemas.value[0] || null
  } catch (e) {
    if (!schemaMounted) return
    toast.error(`加载输入方案失败: ${errorMessage(e)}`)
  }
})

onUnmounted(() => {
  schemaMounted = false
})

/** 点击方案项：切换"使用中"方案 */
async function setActive(schema: RimeSchema) {
  if (currentSchema.value === schema.schema) return
  try {
    await invoke('set_current_schema', { schemaId: schema.schema })
    currentSchema.value = schema.schema
    previewSchema.value = schema
    toast.success(`已切换到 ${schema.name || schema.schema}`)
  } catch (e) {
    toast.error(`切换方案失败: ${errorMessage(e)}`)
  }
}

/** 点击列表项（非切换），仅更新右侧预览 */
function selectPreview(schema: RimeSchema) {
  previewSchema.value = schema
}

function toggleSchema(schema: RimeSchema) {
  schema.enabled = !schema.enabled
}

async function save() {
  try {
    await invoke('save_schemas', { schemas: schemas.value })
    toast.success('方案已保存')
  } catch (e) {
    toast.error(`保存方案失败: ${errorMessage(e)}`)
  }
}

async function importSchema() {
  const selected = await dialogOpen({
    directory: false,
    multiple: false,
    title: '选择输入方案文件',
    filters: [{ name: 'YAML Files', extensions: ['yaml', 'yml'] }],
  })
  if (selected && typeof selected === 'string') {
    try {
      await invoke('import_schema', { filePath: selected })
      const data: SchemaListData = await invoke('get_schemas')
      schemas.value = data.schemas
      currentSchema.value = data.current_schema || ''
      toast.success('输入方案导入成功')
    } catch (e) {
      toast.error(`导入方案失败: ${errorMessage(e)}`)
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
    emitBusEvent(BusEvents.REMOVE_PENDING_DELETE, { delete_type: 'schema', identifier: filename })
  } else {
    pendingDeleteSchemas.value.add(schemaId)
    emitBusEvent(BusEvents.ADD_PENDING_DELETE, {
      delete_type: 'schema',
      identifier: filename,
      label: `方案: ${schemaId}`,
    })
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
        <div class="header-actions">
          <button class="wv-btn wv-btn-primary btn-sm" @click="save">保存配置</button>
          <button class="wv-btn btn-sm" @click="importSchema">📥 导入方案</button>
        </div>
      </div>
      <p class="hint">点击方案可切换为「使用中」，启用/停用、删除后记得保存</p>

      <div class="schema-main">
        <!-- 左侧：方案列表 -->
        <div class="schema-list-col">
          <div
            v-for="schema in schemas"
            :key="schema.schema"
            :class="[
              'schema-item',
              {
                active: currentSchema === schema.schema,
                preview: previewSchema?.schema === schema.schema,
                'is-deleting': pendingDeleteSchemas.has(schema.schema),
              },
            ]"
            @click="selectPreview(schema)"
          >
            <label class="toggle">
              <input
                type="checkbox"
                :checked="schema.enabled"
                @click.stop
                @change="toggleSchema(schema)"
              />
              <span class="toggle-slider"></span>
            </label>
            <span class="schema-id">{{ schema.name || schema.schema }}</span>
            <span
              v-if="currentSchema === schema.schema && !pendingDeleteSchemas.has(schema.schema)"
              class="badge badge-active"
              >使用中</span
            >
            <span v-if="pendingDeleteSchemas.has(schema.schema)" class="badge badge-deleting"
              >待删除</span
            >
            <button
              v-if="pendingDeleteSchemas.has(schema.schema)"
              class="btn-cancel-delete"
              @click.stop="toggleDeleteSchema(schema.schema)"
            >
              取消删除
            </button>
            <button
              v-else
              class="btn-delete"
              :disabled="!canDeleteSchema(schema.schema)"
              :title="!canDeleteSchema(schema.schema) ? '使用中不可删除' : '删除'"
              @click.stop="toggleDeleteSchema(schema.schema)"
            >
              🗑
            </button>
          </div>
        </div>

        <!-- 右侧：方案信息预览 -->
        <div class="schema-preview-col">
          <template v-if="previewSchema">
            <div class="preview-header">
              <h4 class="preview-name">{{ previewSchema.name || previewSchema.schema }}</h4>
              <span v-if="currentSchema === previewSchema.schema" class="badge badge-active"
                >使用中</span
              >
            </div>
            <div class="preview-meta">
              <span v-if="previewSchema.schema" class="meta-item"
                >ID: {{ previewSchema.schema }}</span
              >
              <span v-if="previewSchema.version" class="meta-item"
                >版本: {{ previewSchema.version }}</span
              >
            </div>
            <div v-if="previewSchema.author" class="preview-author">
              作者: {{ previewSchema.author }}
            </div>
            <div v-if="previewSchema.description" class="preview-desc">
              {{ previewSchema.description }}
            </div>
            <div v-if="previewSchema.dependencies.length" class="preview-deps">
              <strong>依赖：</strong>
              <span v-for="d in previewSchema.dependencies" :key="d" class="dep-chip">{{ d }}</span>
            </div>
            <button
              class="wv-btn wv-btn-primary switch-btn"
              :disabled="currentSchema === previewSchema.schema"
              @click="setActive(previewSchema)"
            >
              {{ currentSchema === previewSchema.schema ? '当前使用中' : '切换到此方案' }}
            </button>
          </template>
          <div v-else class="preview-empty">选择左侧方案查看详情</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.schema-manager {
  max-width: 1000px;
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

.header-actions {
  display: flex;
  gap: 8px;
}

.btn-sm {
  padding: 6px 14px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
}

.btn-sm.wv-btn-primary {
  background: var(--color-btn-primary-bg);
  color: var(--color-text-inverse);
  border: none;
}

.hint {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin-bottom: 12px;
}

.schema-main {
  display: grid;
  grid-template-columns: 360px 1fr;
  gap: 16px;
  align-items: start;
}

.schema-list-col {
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
  cursor: pointer;
  transition: border-color 0.15s;
}

.schema-item:hover {
  border-color: var(--color-accent);
}

.schema-item.active {
  border-color: var(--color-success);
  background: var(--color-success-muted);
}

.schema-item.preview {
  outline: 2px solid var(--color-accent-light);
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
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ── 右侧方案信息预览 ─────────────────────────── */
.schema-preview-col {
  padding: 16px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  min-height: 240px;
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 8px;
}

.preview-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.preview-meta {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.preview-author {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin-bottom: 12px;
}

.preview-desc {
  font-size: 13px;
  line-height: 1.6;
  color: var(--color-text-primary);
  white-space: pre-wrap;
  margin-bottom: 12px;
}

.preview-deps {
  font-size: 13px;
  margin-bottom: 16px;
}

.dep-chip {
  display: inline-block;
  padding: 2px 8px;
  margin: 2px;
  background: var(--color-bg-tertiary);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  font-size: 11px;
  color: var(--color-text-secondary);
}

.switch-btn {
  padding: 8px 20px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  background: var(--color-btn-primary-bg);
  color: var(--color-text-inverse);
}

.switch-btn:hover:not(:disabled) {
  background: var(--color-btn-primary-hover);
}

.switch-btn:disabled {
  background: var(--color-border-dark);
  cursor: not-allowed;
}

.preview-empty {
  color: var(--color-text-tertiary);
  font-size: 13px;
  text-align: center;
  padding: 40px 0;
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
