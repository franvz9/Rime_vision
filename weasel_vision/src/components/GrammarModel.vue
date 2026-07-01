<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import DeployNotice from './DeployNotice.vue'
import { useToast } from '../composables/useToast'
import { errorMessage, emitBusEvent, BusEvents } from '../utils'
import WeaselModal from './WeaselModal.vue'

const toast = useToast()

interface GrammarModel {
  filename: string
  file_path: string
  file_size: number
  display_name: string
  formatted_size: string
}

interface SchemaGrammarConfig {
  schema_id: string
  mounted_model: string | null
  collocation_max_length: number
  collocation_min_length: number
  collocation_penalty: number
  non_collocation_penalty: number
  weak_collocation_penalty: number
  rear_penalty: number
  contextual_suggestions: boolean
  max_homophones: number
  max_homographs: number
}

interface GrammarData {
  models: GrammarModel[]
  mount_configs: Record<string, SchemaGrammarConfig>
}

const data = ref<GrammarData | null>(null)
const selectedModel = ref<GrammarModel | null>(null)
const schemaIds = ref<string[]>([])
const showBatchModal = ref(false)
const selectedSchemaIds = ref<Set<string>>(new Set())
const pendingDeleteModels = ref<Set<string>>(new Set())
const batchConfig = ref<SchemaGrammarConfig>({
  schema_id: '',
  mounted_model: null,
  collocation_max_length: 5,
  collocation_min_length: 2,
  collocation_penalty: -16,
  non_collocation_penalty: -8,
  weak_collocation_penalty: -100,
  rear_penalty: -20,
  contextual_suggestions: true,
  max_homophones: 7,
  max_homographs: 7,
})

let gmMounted = true

onMounted(async () => {
  await loadData()
})

onUnmounted(() => {
  gmMounted = false
})

async function loadData() {
  try {
    const schemas = await invoke<{ schemas: Array<{ schema: string }> }>('get_schemas')
    if (!gmMounted) return
    schemaIds.value = schemas.schemas.map((s) => s.schema)
    data.value = await invoke('get_grammar_data', { schemaIds: schemaIds.value })
    if (!gmMounted) return
  } catch (e) {
    if (!gmMounted) return
    toast.error(`加载语言模型数据失败: ${errorMessage(e)}`)
  }
}

async function importGrammar() {
  const selected = await dialogOpen({
    directory: false,
    multiple: false,
    title: '选择语言模型文件',
    filters: [{ name: 'Grammar Files', extensions: ['gram'] }],
  })
  if (selected && typeof selected === 'string') {
    try {
      await invoke('import_grammar', { filePath: selected })
      toast.success('语言模型导入成功')
      await loadData()
    } catch (e) {
      toast.error(`导入语言模型失败: ${errorMessage(e)}`)
    }
  }
}

function selectModel(model: GrammarModel) {
  if (pendingDeleteModels.value.has(model.filename)) return
  selectedModel.value = model
}

function mountedCount(model: GrammarModel): number {
  if (!data.value) return 0
  return Object.values(data.value.mount_configs).filter((c) => c.mounted_model === model.filename)
    .length
}

function isMounted(schemaId: string): boolean {
  if (!data.value) return false
  return data.value.mount_configs[schemaId]?.mounted_model === selectedModel.value?.filename
}

function openBatchModal() {
  if (!selectedModel.value) return
  selectedSchemaIds.value = new Set(schemaIds.value)
  batchConfig.value = {
    schema_id: '',
    mounted_model: selectedModel.value.filename,
    collocation_max_length: 5,
    collocation_min_length: 2,
    collocation_penalty: -16,
    non_collocation_penalty: -8,
    weak_collocation_penalty: -100,
    rear_penalty: -20,
    contextual_suggestions: true,
    max_homophones: 7,
    max_homographs: 7,
  }
  showBatchModal.value = true
}

async function batchMount() {
  const promises = Array.from(selectedSchemaIds.value).map((schemaId) =>
    invoke('mount_grammar', {
      modelFilename: selectedModel.value!.filename,
      schemaId,
      config: { ...batchConfig.value, schema_id: schemaId },
    }).catch((e) => toast.error(`挂载 ${schemaId} 失败: ${errorMessage(e)}`)),
  )
  await Promise.all(promises)
  showBatchModal.value = false
  await loadData()
}

async function batchUnmount() {
  const promises = Array.from(selectedSchemaIds.value).map((schemaId) =>
    invoke('unmount_grammar', { schemaId }).catch((e) =>
      toast.error(`卸载 ${schemaId} 失败: ${errorMessage(e)}`),
    ),
  )
  await Promise.all(promises)
  showBatchModal.value = false
  await loadData()
}

function toggleSchemaId(id: string) {
  if (selectedSchemaIds.value.has(id)) {
    selectedSchemaIds.value.delete(id)
  } else {
    selectedSchemaIds.value.add(id)
  }
  selectedSchemaIds.value = new Set(selectedSchemaIds.value)
}

function canDeleteModel(model: GrammarModel): boolean {
  // Cannot delete a model that is mounted by any schema
  if (mountedCount(model) > 0) return false
  return true
}

function toggleDeleteModel(model: GrammarModel) {
  if (!canDeleteModel(model)) return
  const filename = `${model.filename}.gram`
  if (pendingDeleteModels.value.has(model.filename)) {
    pendingDeleteModels.value.delete(model.filename)
    emitBusEvent(BusEvents.REMOVE_PENDING_DELETE, { delete_type: 'model', identifier: filename })
  } else {
    pendingDeleteModels.value.add(model.filename)
    emitBusEvent(BusEvents.ADD_PENDING_DELETE, {
      delete_type: 'model',
      identifier: filename,
      label: `模型: ${model.display_name}`,
    })
    // If this model is selected, deselect it
    if (selectedModel.value?.filename === model.filename) {
      selectedModel.value = null
    }
  }
  pendingDeleteModels.value = new Set(pendingDeleteModels.value)
}
</script>

<template>
  <div class="grammar-model">
    <DeployNotice />

    <div class="toolbar">
      <button class="wv-btn btn-outline" :disabled="!selectedModel" @click="openBatchModal">
        批量挂载
      </button>
      <button class="wv-btn btn-outline" @click="loadData">刷新</button>
      <button class="wv-btn wv-btn-primary" @click="importGrammar">📥 导入模型</button>
    </div>

    <div class="layout">
      <div class="model-list">
        <h3>可用模型</h3>
        <div v-if="!data || data.models.length === 0" class="wv-empty-state">
          <p>未找到语言模型文件</p>
          <p class="hint">将 .gram 文件放入 Rime 用户目录</p>
        </div>
        <div v-else class="model-items">
          <div
            v-for="model in data.models"
            :key="model.filename"
            :class="[
              'model-item',
              {
                selected: selectedModel?.filename === model.filename,
                'is-deleting': pendingDeleteModels.has(model.filename),
              },
            ]"
            @click="selectModel(model)"
          >
            <div class="model-icon">📦</div>
            <div class="model-info">
              <div class="model-name">
                {{ model.display_name }}
                <span v-if="pendingDeleteModels.has(model.filename)" class="badge badge-deleting"
                  >待删除</span
                >
              </div>
              <div class="model-meta">
                {{ model.formatted_size }}
                <span v-if="mountedCount(model) > 0" class="mounted-count">
                  {{ mountedCount(model) }} 个方案已挂载
                </span>
              </div>
            </div>
            <button
              v-if="pendingDeleteModels.has(model.filename)"
              class="btn-cancel-delete-sm"
              @click.stop="toggleDeleteModel(model)"
            >
              取消
            </button>
            <button
              v-else
              class="btn-delete-sm"
              :disabled="!canDeleteModel(model)"
              :title="!canDeleteModel(model) ? '已挂载不可删除' : '删除'"
              @click.stop="toggleDeleteModel(model)"
            >
              🗑
            </button>
          </div>
        </div>
      </div>

      <div class="model-detail">
        <template v-if="selectedModel">
          <h3>{{ selectedModel.display_name }}</h3>
          <div class="detail-row">
            <span class="label">文件名:</span>
            <span class="value">{{ selectedModel.filename }}.gram</span>
          </div>
          <div class="detail-row">
            <span class="label">大小:</span>
            <span class="value">{{ selectedModel.formatted_size }}</span>
          </div>
          <div class="detail-row">
            <span class="label">路径:</span>
            <span class="value wv-mono">{{ selectedModel.file_path }}</span>
          </div>

          <h4 style="margin-top: 20px">挂载状态</h4>
          <div v-for="schemaId in schemaIds" :key="schemaId" class="mount-row">
            <span class="schema-id">{{ schemaId }}</span>
            <span :class="['mount-status', { mounted: isMounted(schemaId) }]">
              {{ isMounted(schemaId) ? '已挂载' : '未挂载' }}
            </span>
          </div>
        </template>
        <div v-else class="wv-empty-detail">
          <p>选择一个模型查看详情</p>
        </div>
      </div>
    </div>

    <!-- Batch mount modal -->
    <WeaselModal
      :show="showBatchModal"
      :title="`批量挂载 — ${selectedModel?.display_name}`"
      wide
      @close="showBatchModal = false"
    >
      <div class="batch-layout">
        <div class="schema-select">
          <div class="select-header">
            <span>选择方案</span>
            <button class="link-btn" @click="selectedSchemaIds = new Set(schemaIds)">全选</button>
            <button class="link-btn" @click="selectedSchemaIds = new Set()">清除</button>
          </div>
          <div v-for="id in schemaIds" :key="id" class="schema-check">
            <label>
              <input
                type="checkbox"
                :checked="selectedSchemaIds.has(id)"
                @change="toggleSchemaId(id)"
              />
              <span class="wv-mono">{{ id }}</span>
              <span v-if="isMounted(id)" class="mounted-tag">已挂载</span>
            </label>
          </div>
        </div>
        <div class="param-config">
          <h4>参数配置</h4>
          <div class="param-row">
            <label>搭配最大长度</label>
            <input
              v-model.number="batchConfig.collocation_max_length"
              type="number"
              min="3"
              max="10"
            />
          </div>
          <div class="param-row">
            <label>搭配最小长度</label>
            <input
              v-model.number="batchConfig.collocation_min_length"
              type="number"
              min="1"
              max="5"
            />
          </div>
          <div class="param-row">
            <label>搭配惩罚</label>
            <input
              v-model.number="batchConfig.collocation_penalty"
              type="number"
              min="-64"
              max="0"
            />
          </div>
          <div class="param-row">
            <label>非搭配惩罚</label>
            <input
              v-model.number="batchConfig.non_collocation_penalty"
              type="number"
              min="-64"
              max="0"
            />
          </div>
          <div class="param-row">
            <label>弱搭配惩罚</label>
            <input
              v-model.number="batchConfig.weak_collocation_penalty"
              type="number"
              min="-200"
              max="0"
            />
          </div>
          <div class="param-row">
            <label>后置惩罚</label>
            <input v-model.number="batchConfig.rear_penalty" type="number" min="-100" max="0" />
          </div>
          <hr />
          <label class="checkbox">
            <input v-model="batchConfig.contextual_suggestions" type="checkbox" />
            启用上下文建议
          </label>
          <div class="param-row">
            <label>同音词数</label>
            <input v-model.number="batchConfig.max_homophones" type="number" min="1" max="20" />
          </div>
          <div class="param-row">
            <label>同形词数</label>
            <input v-model.number="batchConfig.max_homographs" type="number" min="1" max="20" />
          </div>
        </div>
      </div>
      <template #actions>
        <button
          class="wv-btn wv-btn-primary"
          :disabled="selectedSchemaIds.size === 0"
          @click="batchMount"
        >
          批量挂载
        </button>
        <button
          class="wv-btn btn-outline"
          :disabled="selectedSchemaIds.size === 0"
          @click="batchUnmount"
        >
          批量卸载
        </button>
        <button class="wv-btn" @click="showBatchModal = false">取消</button>
      </template>
    </WeaselModal>
  </div>
</template>

<style scoped>
.grammar-model {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.toolbar {
  display: flex;
  gap: 8px;
}

.layout {
  display: flex;
  gap: 16px;
  min-height: 400px;
}

.model-list {
  width: 280px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
}

.model-list h3 {
  font-size: 14px;
  margin-bottom: 12px;
}

.model-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.model-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  border-radius: 6px;
  cursor: pointer;
}

.model-item:hover {
  background: var(--color-bg-hover);
}

.model-item.selected {
  background: var(--color-accent-light);
}

.model-icon {
  font-size: 20px;
}

.model-name {
  font-size: 13px;
  font-family: monospace;
}

.model-meta {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.mounted-count {
  color: var(--color-success);
}

.model-detail {
  flex: 1;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
}

.detail-row {
  display: flex;
  gap: 8px;
  padding: 4px 0;
  font-size: 13px;
}

.detail-row .label {
  color: var(--color-text-secondary);
  min-width: 60px;
}

.wv-mono {
  font-family: monospace;
  font-size: 12px;
  word-break: break-all;
}

.mount-row {
  display: flex;
  justify-content: space-between;
  padding: 4px 8px;
  font-size: 13px;
  border-radius: 4px;
}

.mount-row:nth-child(odd) {
  background: var(--color-bg-tertiary);
}

.mount-status {
  color: var(--color-text-tertiary);
}

.mount-status.mounted {
  color: var(--color-success);
}

.wv-empty-state,
.wv-empty-detail {
  height: 200px;
}

.hint {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.wv-btn {
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 13px;
}

.btn-outline {
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
}

.link-btn {
  background: none;
  border: none;
  color: var(--color-accent);
  cursor: pointer;
  font-size: 12px;
}

.batch-layout {
  display: flex;
  gap: 16px;
  margin: 16px 0;
}

.schema-select {
  width: 240px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 8px;
  max-height: 300px;
  overflow-y: auto;
}

.select-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.schema-check {
  padding: 4px 0;
}

.schema-check label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  cursor: pointer;
}

.mounted-tag {
  font-size: 11px;
  color: var(--color-success);
}

.param-config {
  flex: 1;
}

.param-config h4 {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.param-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.param-row label {
  font-size: 12px;
  color: var(--color-text-secondary);
  min-width: 100px;
}

.param-row input {
  width: 80px;
  padding: 4px 6px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  font-size: 12px;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
}

.checkbox {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  margin: 8px 0;
}

/* Pending delete styles */
.model-item.is-deleting {
  opacity: 0.5;
  background: var(--color-bg-tertiary);
  border: 1px dashed var(--color-border-dark);
}

.badge-deleting {
  display: inline-block;
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 10px;
  background: var(--color-danger);
  color: white;
  margin-left: 6px;
  vertical-align: middle;
}

.btn-delete-sm {
  background: none;
  border: 1px solid var(--color-danger);
  color: var(--color-danger);
  border-radius: 4px;
  padding: 2px 6px;
  cursor: pointer;
  font-size: 12px;
  margin-left: auto;
}

.btn-delete-sm:hover:not(:disabled) {
  background: var(--color-danger);
  color: white;
}

.btn-delete-sm:disabled {
  color: var(--color-text-tertiary);
  border-color: var(--color-border);
  cursor: not-allowed;
  opacity: 0.5;
}

.btn-cancel-delete-sm {
  background: none;
  border: 1px solid var(--color-success);
  color: var(--color-success);
  border-radius: 4px;
  padding: 2px 6px;
  cursor: pointer;
  font-size: 12px;
  margin-left: auto;
}

.btn-cancel-delete-sm:hover {
  background: var(--color-success);
  color: white;
}
</style>
