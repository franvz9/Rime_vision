<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { formatSize, errorMessage, emitBusEvent, BusEvents } from '../utils'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../composables/useToast'
import WeaselModal from './WeaselModal.vue'

const toast = useToast()

interface UserDictInfo {
  dict_id: string
  display_name: string
  schema_ids: string[]
  entry_count: number
  file_size: number
  last_modified: string
}

interface DictEntry {
  word: string
  code: string
  frequency: number
  commit_count: number
  last_used: string
}

interface DictEntriesResult {
  entries: DictEntry[]
  total: number
  page: number
  per_page: number
  total_frequency: number
}

interface SnapshotInfo {
  file_name: string
  created_at: string
  size: number
  snapshot_type: string
}

const dicts = ref<UserDictInfo[]>([])
const selectedDict = ref<UserDictInfo | null>(null)
const entries = ref<DictEntriesResult | null>(null)
const snapshots = ref<SnapshotInfo[]>([])
const searchQuery = ref('')
const sortBy = ref('frequency_desc')
const currentPage = ref(0)
const perPage = 50
const activeTab = ref<'entries' | 'snapshots'>('entries')
const editingEntry = ref<DictEntry | null>(null)
const editFreq = ref(0)
const showAddModal = ref(false)
const newEntryWord = ref('')
const newEntryCode = ref('')
const newEntryFreq = ref(1)
const isGenerating = ref(false)
const showApplyConfirm = ref(false)
const showSyncNotConfigured = ref(false)

// Request IDs to prevent stale responses from overwriting newer data
let loadEntriesRequestId = 0
let loadSnapshotsRequestId = 0

let dmMounted = true

onMounted(async () => {
  await loadDicts()
})

onUnmounted(() => { dmMounted = false })

async function loadDicts() {
  try {
    dicts.value = await invoke('list_user_dictionaries')
  } catch (e) {
    if (!dmMounted) return
    toast.error(`加载词典列表失败: ${errorMessage(e)}`)
  }
}

async function selectDict(dict: UserDictInfo) {
  selectedDict.value = dict
  currentPage.value = 0
  activeTab.value = 'entries'
  await loadEntries()
  await loadSnapshots()
}

async function loadEntries() {
  if (!selectedDict.value) return

  // Increment request ID to discard stale responses
  const requestId = ++loadEntriesRequestId

  try {
    const result = await invoke<DictEntriesResult>('load_user_dict_entries', {
      dictId: selectedDict.value.dict_id,
      page: currentPage.value,
      perPage,
      sortBy: sortBy.value,
      search: searchQuery.value || null,
    })

    // Only update if this request is still the latest
    if (requestId === loadEntriesRequestId) {
      entries.value = result
    }
  } catch (e) {
    toast.error(`加载词条失败: ${errorMessage(e)}`)
  }
}

async function loadSnapshots() {
  if (!selectedDict.value) return

  // Increment request ID to discard stale responses
  const requestId = ++loadSnapshotsRequestId

  try {
    const result = await invoke<SnapshotInfo[]>('list_snapshots', {
      dictId: selectedDict.value.dict_id,
    })

    // Only update if this request is still the latest
    if (requestId === loadSnapshotsRequestId) {
      snapshots.value = result
      // Update entry_count based on actual snapshot existence
      // If we have snapshots, set entry_count to the total from loaded entries
      // If no snapshots, keep it as -1 (needs sync)
      if (result.length > 0 && entries.value) {
        selectedDict.value = {
          ...selectedDict.value,
          entry_count: entries.value.total
        }
      }
    }
  } catch (e) {
    toast.error(`加载快照失败: ${errorMessage(e)}`)
  }
}

async function generateSnapshot() {
  if (!selectedDict.value || isGenerating.value) return
  isGenerating.value = true
  try {
    await invoke('create_snapshot', { dictId: selectedDict.value.dict_id })
    await loadEntries()
    await loadSnapshots()
  } catch (e: unknown) {
    const errorMsg = String(e)
    
    if (errorMsg.includes('SYNC_NOT_CONFIGURED')) {
      showSyncNotConfigured.value = true
    } else {
      toast.error(`生成快照失败: ${errorMessage(e)}`)
    }
  } finally {
    isGenerating.value = false
  }
}

function goToSyncSettings() {
  showSyncNotConfigured.value = false
  emitBusEvent(BusEvents.NAVIGATE_TO_SYNC_SETTINGS, undefined)
}

async function applySnapshot() {
  if (!selectedDict.value || snapshots.value.length === 0) return
  showApplyConfirm.value = true
}

async function confirmApplySnapshot() {
  showApplyConfirm.value = false
  if (!selectedDict.value || snapshots.value.length === 0) return
  
  try {
    await invoke('apply_modified_snapshot', {
      dictId: selectedDict.value.dict_id,
      fileName: snapshots.value[0].file_name
    })
    toast.success('修改已应用！Rime 正在重新部署...')
    setTimeout(async () => {
      await loadEntries()
      await loadSnapshots()
    }, 2000)
  } catch (e) {
    toast.error(`应用修改失败: ${errorMessage(e)}`)
  }
}

function prevPage() {
  if (currentPage.value > 0) {
    currentPage.value--
    loadEntries()
  }
}

function nextPage() {
  if (entries.value && (currentPage.value + 1) * perPage < entries.value.total) {
    currentPage.value++
    loadEntries()
  }
}

function formatFreq(n: number): string {
  return n.toLocaleString()
}

async function onSearch() {
  currentPage.value = 0
  await loadEntries()
}

async function onSortChange() {
  currentPage.value = 0
  await loadEntries()
}

const showBatchDelete = ref(false)
const batchThreshold = ref(10)
const showClearConfirm = ref(false)
const showExportResult = ref(false)
const exportCount = ref(0)

async function batchDelete() {
  if (!selectedDict.value) return
  try {
    await invoke('batch_delete_low_frequency', {
      dictId: selectedDict.value.dict_id,
      threshold: batchThreshold.value,
    })
    showBatchDelete.value = false
    await loadEntries()
    if (selectedDict.value && entries.value) {
      selectedDict.value = { ...selectedDict.value, entry_count: entries.value.total }
    }
  } catch (e) {
    toast.error(`批量删除失败: ${errorMessage(e)}`)
  }
}

async function exportDict() {
  if (!selectedDict.value) return
  try {
    const path = await invoke('plugin:dialog|save', {
      defaultPath: `${selectedDict.value.dict_id}_export.txt`,
      filters: [{ name: 'Text', extensions: ['txt'] }],
    })
    if (path) {
      const count = await invoke('export_user_dict', {
        dictId: selectedDict.value.dict_id,
        outputPath: path,
      })
      exportCount.value = count as number
      showExportResult.value = true
    }
  } catch (e) {
    toast.error(`导出失败: ${errorMessage(e)}`)
  }
}

async function clearDict() {
  if (!selectedDict.value) return
  try {
    await invoke('clear_user_dict', { dictId: selectedDict.value.dict_id })
    showClearConfirm.value = false
    await loadEntries()
    if (selectedDict.value) {
      selectedDict.value = { ...selectedDict.value, entry_count: 0 }
    }
  } catch (e) {
    toast.error(`清空词典失败: ${errorMessage(e)}`)
  }
}

function startEdit(entry: DictEntry) {
  editingEntry.value = entry
  editFreq.value = entry.frequency
}

function cancelEdit() {
  editingEntry.value = null
  editFreq.value = 0
}

async function saveEdit() {
  if (!selectedDict.value || !editingEntry.value) return
  try {
    await invoke('update_entry_frequency', {
      dictId: selectedDict.value.dict_id,
      word: editingEntry.value.word,
      code: editingEntry.value.code,
      newFreq: editFreq.value,
    })
    cancelEdit()
    await loadEntries()
  } catch (e) {
    toast.error(`更新词条失败: ${errorMessage(e)}`)
  }
}

async function deleteEntry(entry: DictEntry) {
  if (!selectedDict.value) return
  try {
    await invoke('delete_entries', {
      dictId: selectedDict.value.dict_id,
      entriesToDelete: [{ word: entry.word, code: entry.code }],
    })
    await loadEntries()
    if (selectedDict.value) {
      selectedDict.value = { ...selectedDict.value, entry_count: entries.value?.total || 0 }
    }
  } catch (e) {
    toast.error(`删除词条失败: ${errorMessage(e)}`)
  }
}

async function addNewEntry() {
  if (!selectedDict.value || !newEntryWord.value || !newEntryCode.value) return
  try {
    await invoke('add_dict_entry', {
      dictId: selectedDict.value.dict_id,
      entry: {
        word: newEntryWord.value,
        code: newEntryCode.value,
        frequency: newEntryFreq.value,
      },
    })
    showAddModal.value = false
    newEntryWord.value = ''
    newEntryCode.value = ''
    newEntryFreq.value = 1
    await loadEntries()
    if (selectedDict.value && entries.value) {
      selectedDict.value = { ...selectedDict.value, entry_count: entries.value.total }
    }
  } catch (e) {
    toast.error(`添加词条失败: ${errorMessage(e)}`)
  }
}
</script>

<template>
  <div class="dict-manager">
    <div class="layout">
      <div class="dict-list">
        <h3>用户词典</h3>
        <div v-if="dicts.length === 0" class="empty-state">
          <p>未找到用户词典</p>
          <p class="hint">使用输入法打字后自动生成</p>
        </div>
        <div v-else class="dict-items">
          <div
            v-for="dict in dicts"
            :key="dict.dict_id"
            :class="['dict-item', { selected: selectedDict?.dict_id === dict.dict_id }]"
            @click="selectDict(dict)"
          >
            <div class="dict-icon">📖</div>
            <div class="dict-info">
              <div class="dict-name">{{ dict.display_name }}</div>
              <div class="dict-meta">
                            <template v-if="dict.entry_count < 0">
                              <span class="hint" title="词典数据存在但未导出为文本快照。请在 Rime 输入法中执行「同步用户资料」以生成快照。">需同步生成快照</span> · {{ formatSize(dict.file_size) }}
                            </template>
                            <template v-else>
                              {{ dict.entry_count.toLocaleString() }} 词条 · {{ formatSize(dict.file_size) }}
                            </template>
                          </div>
            </div>
          </div>
        </div>
      </div>

      <div class="dict-detail">
        <template v-if="selectedDict">
          <div class="detail-header">
            <h3>{{ selectedDict.display_name }}</h3>
            <div class="tabs">
              <button :class="['tab', { active: activeTab === 'entries' }]" @click="activeTab = 'entries'">词条</button>
              <button :class="['tab', { active: activeTab === 'snapshots' }]" @click="activeTab = 'snapshots'">快照</button>
            </div>
          </div>

          <!-- Entries tab -->
          <div v-if="activeTab === 'entries'">
            <!-- Show generate snapshot button when no snapshot exists -->
            <div v-if="snapshots.length === 0" class="snapshot-hint">
              <p>📭 此词典尚未生成文本快照</p>
                        
              <!-- Check if userdb directory has any data -->
              <template v-if="selectedDict.file_size > 1024">
                <p class="hint-text">检测到用户词典数据存在（{{ formatSize(selectedDict.file_size) }}），但尚未导出为文本格式。</p>
                <button class="btn btn-primary" :disabled="isGenerating" @click="generateSnapshot">
                  {{ isGenerating ? '生成中...' : '🔄 触发同步并生成快照' }}
                </button>
                <p class="hint-text small">点击后将触发 Rime 同步，将用户词典导出到同步目录。</p>
              </template>
              <template v-else>
                <p class="hint-text">⚠️ 此方案下尚未产生任何用户词条。</p>
                <p class="hint-text">请先使用该输入法方案打字，Rime 会自动记录您输入过的词语，然后执行「同步用户资料」即可看到词条。</p>
              </template>
            </div>

            <template v-else>
            <div class="toolbar">
              <input
                v-model="searchQuery"
                placeholder="搜索编码或文字..."
                class="search-input"
                @keyup.enter="onSearch"
              />
              <select v-model="sortBy" @change="onSortChange" class="sort-select">
                <option value="frequency_desc">词频 ↓</option>
                <option value="frequency_asc">词频 ↑</option>
                <option value="word_asc">文字 ↑</option>
                <option value="code_asc">编码 ↑</option>
              </select>
              <button class="btn btn-primary" @click="showAddModal = true">+ 新增词条</button>
            </div>

            <div v-if="entries" class="entries-table">
              <!-- Show hint when no snapshot available -->
              <div v-if="selectedDict.entry_count < 0 && entries.entries.length === 0" class="empty-state">
                <p>📭 此词典尚未生成文本快照</p>
                <p class="hint-text">请在 Rime 输入法中执行「同步用户资料」以生成快照，然后即可查看和编辑词条。</p>
                <p class="hint-text small">提示：点击系统托盘的 Rime 图标 → 选择「同步用户资料」</p>
              </div>
              
              <!-- Show hint when snapshot exists but is empty -->
              <div v-else-if="entries.entries.length === 0" class="empty-state">
                <p>📭 快照文件中没有词条数据</p>
                <p class="hint-text">这可能是因为：</p>
                <ul class="hint-list">
                  <li>您还没有使用这个方案打字（Rime 只会记录实际输入过的词）</li>
                  <li>或者需要先在 Rime 中执行「同步用户资料」来导出当前数据</li>
                </ul>
                <button class="btn btn-primary" :disabled="isGenerating" @click="generateSnapshot">
                  {{ isGenerating ? '生成中...' : '🔄 重新同步' }}
                </button>
              </div>
              
              <template v-else>
              <div class="table-header">
                <span class="col-idx">#</span>
                <span class="col-word">文字</span>
                <span class="col-code">编码</span>
                <span class="col-freq">词频</span>
                <span class="col-actions">操作</span>
              </div>
              <div v-for="(entry, idx) in entries.entries" :key="entry.word + entry.code" class="table-row">
                <template v-if="editingEntry?.word === entry.word && editingEntry?.code === entry.code">
                  <span class="col-idx">{{ entries.page * perPage + idx + 1 }}</span>
                  <span class="col-word">{{ entry.word }}</span>
                  <span class="col-code">{{ entry.code }}</span>
                  <span class="col-freq">
                    <input type="number" v-model.number="editFreq" class="freq-input" min="0" />
                  </span>
                  <span class="col-actions">
                    <button class="icon-btn save" @click="saveEdit">✓</button>
                    <button class="icon-btn" @click="cancelEdit">✕</button>
                  </span>
                </template>
                <template v-else>
                  <span class="col-idx">{{ entries.page * perPage + idx + 1 }}</span>
                  <span class="col-word">{{ entry.word }}</span>
                  <span class="col-code">{{ entry.code }}</span>
                  <span class="col-freq">{{ formatFreq(entry.frequency) }}</span>
                  <span class="col-actions">
                    <button class="icon-btn" @click="startEdit(entry)">📝</button>
                    <button class="icon-btn danger" @click="deleteEntry(entry)">🗑</button>
                  </span>
                </template>
              </div>
              </template>
            </div>

            <div v-if="entries && entries.entries.length > 0" class="pagination">
              <span class="page-info">
                共 {{ entries.total.toLocaleString() }} 词条 · 总词频 {{ formatFreq(entries.total_frequency) }}
              </span>
              <div class="page-buttons">
                <button class="btn btn-sm" @click="prevPage" :disabled="currentPage === 0">上一页</button>
                <span class="page-num">{{ currentPage + 1 }} / {{ Math.ceil(entries.total / perPage) }}</span>
                <button class="btn btn-sm" @click="nextPage" :disabled="(currentPage + 1) * perPage >= entries.total">下一页</button>
              </div>
            </div>

            <div class="batch-actions">
              <button class="btn btn-sm" @click="showBatchDelete = true">批量删除低频</button>
              <button class="btn btn-sm" @click="exportDict">导出码表</button>
              <button class="btn btn-sm danger" @click="showClearConfirm = true">清空词典</button>
            </div>
            </template>
          </div>

          <!-- Snapshots tab -->
          <div v-if="activeTab === 'snapshots'">
            <div v-if="snapshots.length === 0" class="empty-state">
              <p>📭 暂无快照</p>
              <p class="hint-text">请先在「词条」标签页生成快照</p>
            </div>
            <div v-else class="snapshot-info">
              <div class="snapshot-card">
                <div class="card-header">
                  <span class="snap-icon">📄</span>
                  <div class="card-title">
                    <strong>{{ snapshots[0].file_name }}</strong>
                    <span class="badge badge-sync">{{ snapshots[0].snapshot_type }}</span>
                  </div>
                </div>
                <div class="card-details">
                  <div class="detail-row">
                    <span class="label">创建时间:</span>
                    <span>{{ snapshots[0].created_at }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="label">文件大小:</span>
                    <span>{{ formatSize(snapshots[0].size) }}</span>
                  </div>
                </div>
                <div class="card-actions">
                  <button class="btn btn-primary" @click="applySnapshot">
                    ✅ 应用修改
                  </button>
                </div>
              </div>
              <div class="hint-box">
                <p>💡 提示：</p>
                <ul>
                  <li>此快照来自 Rime 同步目录，是最新的用户词典数据</li>
                  <li>您可以在「词条」标签页中编辑词条（新增、删除、修改词频）</li>
                  <li>编辑完成后点击「应用修改」，将更新写回 Rime 用户词典</li>
                  <li><strong style="color: var(--color-danger);">⚠️ 应用后会自动触发 Rime 重新部署，使更改生效</strong></li>
                  <li>如需获取最新快照，请前往「同步管理」页面执行完整同步</li>
                </ul>
              </div>
            </div>
          </div>
        </template>
        <div v-else class="empty-detail">
          <p>选择一个词典查看详情</p>
        </div>
      </div>
    </div>

    <!-- Batch delete modal -->
    <WeaselModal :show="showBatchDelete" title="批量删除低频词条" @close="showBatchDelete = false">
      <p class="hint">删除词频低于阈值的所有词条</p>
      <div class="form-group">
        <label>词频阈值:</label>
        <input type="number" v-model.number="batchThreshold" min="1" class="input" />
      </div>
      <template #actions>
        <button class="btn" @click="showBatchDelete = false">取消</button>
        <button class="btn btn-danger" @click="batchDelete">删除</button>
      </template>
    </WeaselModal>

    <!-- Clear confirm modal -->
    <WeaselModal :show="showClearConfirm" title="清空词典" @close="showClearConfirm = false">
      <p class="warning">此操作将删除「{{ selectedDict?.display_name }}」的所有词条，不可撤销！</p>
      <template #actions>
        <button class="btn" @click="showClearConfirm = false">取消</button>
        <button class="btn btn-danger" @click="clearDict">确认清空</button>
      </template>
    </WeaselModal>

    <!-- Export result modal -->
    <WeaselModal :show="showExportResult" title="导出完成" @close="showExportResult = false">
      <p>已导出 {{ exportCount.toLocaleString() }} 个词条</p>
      <template #actions>
        <button class="btn btn-primary" @click="showExportResult = false">确定</button>
      </template>
    </WeaselModal>

    <!-- Add entry modal -->
    <WeaselModal :show="showAddModal" title="新增词条" @close="showAddModal = false">
      <div class="form-group">
        <label>文字:</label>
        <input v-model="newEntryWord" class="input" placeholder="输入汉字" />
      </div>
      <div class="form-group">
        <label>编码:</label>
        <input v-model="newEntryCode" class="input" placeholder="如：nihao" />
      </div>
      <div class="form-group">
        <label>初始词频:</label>
        <input type="number" v-model.number="newEntryFreq" class="input" min="1" />
      </div>
      <template #actions>
        <button class="btn" @click="showAddModal = false">取消</button>
        <button class="btn btn-primary" @click="addNewEntry" :disabled="!newEntryWord || !newEntryCode">添加</button>
      </template>
    </WeaselModal>

    <!-- Apply snapshot confirm modal -->
    <WeaselModal :show="showApplyConfirm" title="确认应用修改" @close="showApplyConfirm = false">
      <p>应用修改后，将执行以下操作：</p>
      <ul style="padding-left: 20px; margin: 8px 0; font-size: 13px; color: var(--color-text-secondary);">
        <li>将修改后的词条数据写回 Rime 用户词典</li>
        <li>触发 Rime 重新部署（需要几秒时间）</li>
      </ul>
      <template #actions>
        <button class="btn" @click="showApplyConfirm = false">取消</button>
        <button class="btn btn-primary" @click="confirmApplySnapshot">确认应用</button>
      </template>
    </WeaselModal>

    <!-- Sync not configured modal -->
    <WeaselModal :show="showSyncNotConfigured" title="需要配置同步目录" @close="showSyncNotConfigured = false">
      <p>生成快照需要先配置同步目录。</p>
      <p style="font-size: 13px; color: var(--color-text-secondary); margin-top: 8px;">
        Rime 的同步功能会将用户词典和配置导出到指定文件夹，
        配合 iCloud、WebDAV、坚果云等文件同步服务可实现多设备同步。
      </p>
      <p style="font-size: 13px; color: var(--color-text-secondary); margin-top: 4px;">
        是否立即前往同步设置？
      </p>
      <template #actions>
        <button class="btn" @click="showSyncNotConfigured = false">取消</button>
        <button class="btn btn-primary" @click="goToSyncSettings">前往设置</button>
      </template>
    </WeaselModal>
  </div>
</template>

<style scoped>
.dict-manager {
  display: flex;
  flex-direction: column;
}

.layout {
  display: flex;
  gap: 16px;
  min-height: 400px;
}

.dict-list {
  width: 260px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 12px;
}

.dict-list h3 {
  font-size: 14px;
  margin-bottom: 12px;
}

.dict-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.dict-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  border-radius: 6px;
  cursor: pointer;
}

.dict-item:hover {
  background: var(--color-bg-hover);
}

.dict-item.selected {
  background: var(--color-accent-light);
}

.dict-icon {
  font-size: 20px;
}

.dict-info {
  flex: 1;
}

.dict-name {
  font-size: 13px;
  font-weight: 500;
}

.dict-meta {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.dict-detail {
  flex: 1;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.detail-header h3 {
  font-size: 15px;
  margin: 0;
}

.tabs {
  display: flex;
  gap: 4px;
  background: var(--color-bg-active);
  padding: 3px;
  border-radius: 6px;
}

.tab {
  padding: 4px 12px;
  border: none;
  background: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--color-text-primary);
}

.tab.active {
  background: var(--color-bg-secondary);
  box-shadow: var(--shadow-sm);
}

.toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.search-input {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 13px;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
}

.sort-select {
  padding: 6px 10px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 13px;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
}

.entries-table {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

.table-header {
  display: flex;
  padding: 8px 12px;
  background: var(--color-bg-tertiary);
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.table-row {
  display: flex;
  padding: 6px 12px;
  font-size: 13px;
  border-top: 1px solid var(--color-border-light);
}

.table-row:hover {
  background: var(--color-bg-hover);
}

.col-idx {
  width: 50px;
  color: var(--color-text-tertiary);
}

.col-word {
  flex: 1;
  font-weight: 500;
}

.col-code {
  flex: 1;
  font-family: monospace;
  color: var(--color-text-secondary);
}

.col-freq {
  width: 80px;
  text-align: right;
  font-family: monospace;
}

.col-actions {
  width: 70px;
  text-align: right;
}

.freq-input {
  width: 70px;
  padding: 2px 4px;
  border: 1px solid var(--color-accent);
  border-radius: 3px;
  font-size: 12px;
  text-align: right;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
}

.icon-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  padding: 2px 4px;
}

.icon-btn.danger {
  color: var(--color-danger);
}

.icon-btn.save {
  color: var(--color-success);
}

.pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 12px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.page-buttons {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page-num {
  font-size: 12px;
  min-width: 60px;
  text-align: center;
}

.btn {
  padding: 4px 10px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.snapshot-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.snapshot-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  font-size: 13px;
  background: var(--color-bg-tertiary);
  border-radius: 4px;
}

.snap-icon {
  font-size: 14px;
}

.snap-name {
  flex: 1;
  font-family: monospace;
  font-size: 12px;
}

.snap-type {
  font-size: 11px;
  padding: 1px 6px;
  background: var(--color-accent-light);
  border-radius: 8px;
  color: var(--color-accent);
}

.snap-time {
  color: var(--color-text-tertiary);
  font-size: 12px;
}

.snap-size {
  color: var(--color-text-tertiary);
  font-size: 12px;
}

.snapshot-item .icon-btn {
  margin-left: auto;
}

.empty-state,
.empty-detail {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--color-text-tertiary);
}

.hint {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.batch-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--color-border);
}

.btn.danger {
  color: var(--color-danger);
  border-color: var(--color-danger);
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-bg-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: var(--color-bg-modal);
  border-radius: 12px;
  padding: 24px;
  width: 400px;
}

.modal h3 {
  margin-bottom: 8px;
}

.form-group {
  margin-bottom: 12px;
}

.form-group label {
  display: block;
  font-size: 13px;
  margin-bottom: 4px;
}

.input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 14px;
}

.warning {
  color: var(--color-danger);
  font-size: 14px;
  margin: 12px 0;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}

.btn-primary {
  background: var(--color-accent);
  color: white;
  border: none;
}

.btn-danger {
  background: var(--color-danger);
  color: white;
  border: none;
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--color-text-secondary);
}

.empty-state p {
  margin: 8px 0;
}

.hint-text {
  font-size: 13px;
  color: var(--color-text-tertiary);
  line-height: 1.5;
}

.hint-text.small {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.hint-list {
  text-align: left;
  display: inline-block;
  margin: 12px auto;
  padding-left: 20px;
  color: var(--color-text-tertiary);
  font-size: 13px;
}

.hint-list li {
  margin-bottom: 6px;
}

.snapshot-hint {
  text-align: center;
  padding: 40px 20px;
  border: 2px dashed var(--color-border);
  border-radius: 8px;
  margin-bottom: 16px;
}

.snapshot-info {
  padding: 20px;
}

.snapshot-card {
  background: var(--color-bg-tertiary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.badge-sync {
  background: var(--color-accent-light);
  color: var(--color-accent);
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.card-details {
  margin-bottom: 12px;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  font-size: 13px;
}

.detail-row .label {
  color: var(--color-text-secondary);
}

.card-actions {
  display: flex;
  gap: 8px;
}

.hint-box {
  background: var(--color-warning-light);
  border-left: 4px solid var(--color-warning);
  padding: 12px 16px;
  border-radius: 4px;
}

.hint-box p {
  margin: 0 0 8px 0;
  font-weight: 500;
  color: var(--color-warning-dark);
}

.hint-box ul {
  margin: 0;
  padding-left: 20px;
  color: var(--color-warning-dark);
  font-size: 13px;
}

.hint-box li {
  margin-bottom: 4px;
}

.snapshot-hint p {
  margin: 8px 0;
  color: var(--color-text-secondary);
}

.snapshot-hint .btn-primary {
  margin: 16px 0;
  padding: 10px 20px;
  font-size: 14px;
}
</style>
