<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['changed'])

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

onMounted(async () => {
  await loadData()
})

async function loadData() {
  try {
    const schemas: any = await invoke('get_schemas')
    schemaIds.value = schemas.schemas.map((s: any) => s.schema)
    data.value = await invoke('get_grammar_data', { schemaIds: schemaIds.value })
  } catch (e) {
    console.error('Failed to load grammar data:', e)
  }
}

function selectModel(model: GrammarModel) {
  selectedModel.value = model
}

function mountedCount(model: GrammarModel): number {
  if (!data.value) return 0
  return Object.values(data.value.mount_configs).filter(
    (c) => c.mounted_model === model.filename
  ).length
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
  for (const schemaId of selectedSchemaIds.value) {
    try {
      await invoke('mount_grammar', {
        modelFilename: selectedModel.value!.filename,
        schemaId,
        config: { ...batchConfig.value, schema_id: schemaId },
      })
    } catch (e) {
      console.error('Mount failed:', e)
    }
  }
  showBatchModal.value = false
  await loadData()
  emit('changed')
}

async function batchUnmount() {
  for (const schemaId of selectedSchemaIds.value) {
    try {
      await invoke('unmount_grammar', { schemaId })
    } catch (e) {
      console.error('Unmount failed:', e)
    }
  }
  showBatchModal.value = false
  await loadData()
  emit('changed')
}

function toggleSchemaId(id: string) {
  if (selectedSchemaIds.value.has(id)) {
    selectedSchemaIds.value.delete(id)
  } else {
    selectedSchemaIds.value.add(id)
  }
  selectedSchemaIds.value = new Set(selectedSchemaIds.value)
}
</script>

<template>
  <div class="grammar-model">
    <div class="toolbar">
      <button class="btn btn-outline" @click="openBatchModal" :disabled="!selectedModel">批量挂载</button>
      <button class="btn btn-outline" @click="loadData">刷新</button>
    </div>

    <div class="layout">
      <div class="model-list">
        <h3>可用模型</h3>
        <div v-if="!data || data.models.length === 0" class="empty-state">
          <p>未找到语言模型文件</p>
          <p class="hint">将 .gram 文件放入 Rime 用户目录</p>
        </div>
        <div v-else class="model-items">
          <div
            v-for="model in data.models"
            :key="model.filename"
            :class="['model-item', { selected: selectedModel?.filename === model.filename }]"
            @click="selectModel(model)"
          >
            <div class="model-icon">📦</div>
            <div class="model-info">
              <div class="model-name">{{ model.display_name }}</div>
              <div class="model-meta">
                {{ model.formatted_size }}
                <span v-if="mountedCount(model) > 0" class="mounted-count">
                  {{ mountedCount(model) }} 个方案已挂载
                </span>
              </div>
            </div>
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
            <span class="value mono">{{ selectedModel.file_path }}</span>
          </div>

          <h4 style="margin-top: 20px">挂载状态</h4>
          <div v-for="schemaId in schemaIds" :key="schemaId" class="mount-row">
            <span class="schema-id">{{ schemaId }}</span>
            <span :class="['mount-status', { mounted: isMounted(schemaId) }]">
              {{ isMounted(schemaId) ? '已挂载' : '未挂载' }}
            </span>
          </div>
        </template>
        <div v-else class="empty-detail">
          <p>选择一个模型查看详情</p>
        </div>
      </div>
    </div>

    <!-- Batch mount modal -->
    <div v-if="showBatchModal" class="modal-overlay" @click.self="showBatchModal = false">
      <div class="modal">
        <h3>批量挂载 — {{ selectedModel?.display_name }}</h3>
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
                <span class="mono">{{ id }}</span>
                <span v-if="isMounted(id)" class="mounted-tag">已挂载</span>
              </label>
            </div>
          </div>
          <div class="param-config">
            <h4>参数配置</h4>
            <div class="param-row">
              <label>搭配最大长度</label>
              <input type="number" v-model.number="batchConfig.collocation_max_length" min="3" max="10" />
            </div>
            <div class="param-row">
              <label>搭配最小长度</label>
              <input type="number" v-model.number="batchConfig.collocation_min_length" min="1" max="5" />
            </div>
            <div class="param-row">
              <label>搭配惩罚</label>
              <input type="number" v-model.number="batchConfig.collocation_penalty" min="-64" max="0" />
            </div>
            <div class="param-row">
              <label>非搭配惩罚</label>
              <input type="number" v-model.number="batchConfig.non_collocation_penalty" min="-64" max="0" />
            </div>
            <div class="param-row">
              <label>弱搭配惩罚</label>
              <input type="number" v-model.number="batchConfig.weak_collocation_penalty" min="-200" max="0" />
            </div>
            <div class="param-row">
              <label>后置惩罚</label>
              <input type="number" v-model.number="batchConfig.rear_penalty" min="-100" max="0" />
            </div>
            <hr />
            <label class="checkbox">
              <input type="checkbox" v-model="batchConfig.contextual_suggestions" />
              启用上下文建议
            </label>
            <div class="param-row">
              <label>同音词数</label>
              <input type="number" v-model.number="batchConfig.max_homophones" min="1" max="20" />
            </div>
            <div class="param-row">
              <label>同形词数</label>
              <input type="number" v-model.number="batchConfig.max_homographs" min="1" max="20" />
            </div>
          </div>
        </div>
        <div class="modal-actions">
          <button class="btn btn-primary" @click="batchMount" :disabled="selectedSchemaIds.size === 0">批量挂载</button>
          <button class="btn btn-outline" @click="batchUnmount" :disabled="selectedSchemaIds.size === 0">批量卸载</button>
          <button class="btn" @click="showBatchModal = false">取消</button>
        </div>
      </div>
    </div>
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
  background: white;
  border: 1px solid #e5e5e5;
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
  background: #f5f5f5;
}

.model-item.selected {
  background: #e3f2fd;
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
  color: #999;
}

.mounted-count {
  color: #34c759;
}

.model-detail {
  flex: 1;
  background: white;
  border: 1px solid #e5e5e5;
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
  color: #666;
  min-width: 60px;
}

.mono {
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
  background: #f9f9f9;
}

.mount-status {
  color: #999;
}

.mount-status.mounted {
  color: #34c759;
}

.empty-state,
.empty-detail {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #999;
}

.hint {
  font-size: 12px;
  color: #ccc;
}

.btn {
  padding: 6px 14px;
  border: 1px solid #ddd;
  background: white;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.btn-primary {
  background: #007aff;
  color: white;
  border: none;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-outline {
  background: white;
}

.link-btn {
  background: none;
  border: none;
  color: #007aff;
  cursor: pointer;
  font-size: 12px;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: white;
  border-radius: 12px;
  padding: 24px;
  width: 600px;
  max-height: 80vh;
  overflow-y: auto;
}

.batch-layout {
  display: flex;
  gap: 16px;
  margin: 16px 0;
}

.schema-select {
  width: 240px;
  border: 1px solid #e5e5e5;
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
  color: #666;
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
  color: #34c759;
}

.param-config {
  flex: 1;
}

.param-config h4 {
  font-size: 13px;
  color: #666;
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
  color: #666;
  min-width: 100px;
}

.param-row input {
  width: 80px;
  padding: 4px 6px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 12px;
}

.checkbox {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  margin: 8px 0;
}

.modal-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
</style>
