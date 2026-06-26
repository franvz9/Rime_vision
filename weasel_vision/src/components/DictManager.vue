<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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

onMounted(async () => {
  await loadDicts()
})

async function loadDicts() {
  try {
    dicts.value = await invoke('list_user_dictionaries')
  } catch (e) {
    console.error('Failed to load dicts:', e)
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
  try {
    entries.value = await invoke('load_user_dict_entries', {
      dictId: selectedDict.value.dict_id,
      page: currentPage.value,
      perPage,
      sortBy: sortBy.value,
      search: searchQuery.value || null,
    })
  } catch (e) {
    console.error('Failed to load entries:', e)
  }
}

async function loadSnapshots() {
  if (!selectedDict.value) return
  try {
    snapshots.value = await invoke('list_snapshots', {
      dictId: selectedDict.value.dict_id,
    })
  } catch (e) {
    console.error('Failed to load snapshots:', e)
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

function formatSize(bytes: number): string {
  if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${bytes} B`
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
    console.error('Batch delete failed:', e)
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
    console.error('Export failed:', e)
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
    console.error('Clear failed:', e)
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
    console.error('Failed to update entry:', e)
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
    console.error('Failed to delete entry:', e)
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
              <div class="dict-meta">{{ dict.entry_count.toLocaleString() }} 词条 · {{ formatSize(dict.file_size) }}</div>
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
            </div>

            <div v-if="entries" class="entries-table">
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
                    <button class="icon-btn" @click="startEdit(entry)">✏</button>
                    <button class="icon-btn danger" @click="deleteEntry(entry)">🗑</button>
                  </span>
                </template>
              </div>
            </div>

            <div v-if="entries" class="pagination">
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
          </div>

          <!-- Snapshots tab -->
          <div v-if="activeTab === 'snapshots'">
            <div v-if="snapshots.length === 0" class="empty-state">
              <p>暂无快照</p>
            </div>
            <div v-else class="snapshot-list">
              <div v-for="snap in snapshots" :key="snap.file_name" class="snapshot-item">
                <span class="snap-icon">📄</span>
                <span class="snap-name">{{ snap.file_name }}</span>
                <span class="snap-type">{{ snap.snapshot_type }}</span>
                <span class="snap-time">{{ snap.created_at }}</span>
                <span class="snap-size">{{ formatSize(snap.size) }}</span>
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
    <div v-if="showBatchDelete" class="modal-overlay" @click.self="showBatchDelete = false">
      <div class="modal">
        <h3>批量删除低频词条</h3>
        <p class="hint">删除词频低于阈值的所有词条</p>
        <div class="form-group">
          <label>词频阈值:</label>
          <input type="number" v-model.number="batchThreshold" min="1" class="input" />
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showBatchDelete = false">取消</button>
          <button class="btn btn-danger" @click="batchDelete">删除</button>
        </div>
      </div>
    </div>

    <!-- Clear confirm modal -->
    <div v-if="showClearConfirm" class="modal-overlay" @click.self="showClearConfirm = false">
      <div class="modal">
        <h3>清空词典</h3>
        <p class="warning">此操作将删除「{{ selectedDict?.display_name }}」的所有词条，不可撤销！</p>
        <div class="modal-actions">
          <button class="btn" @click="showClearConfirm = false">取消</button>
          <button class="btn btn-danger" @click="clearDict">确认清空</button>
        </div>
      </div>
    </div>

    <!-- Export result modal -->
    <div v-if="showExportResult" class="modal-overlay" @click.self="showExportResult = false">
      <div class="modal">
        <h3>导出完成</h3>
        <p>已导出 {{ exportCount.toLocaleString() }} 个词条</p>
        <div class="modal-actions">
          <button class="btn btn-primary" @click="showExportResult = false">确定</button>
        </div>
      </div>
    </div>
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
  background: white;
  border: 1px solid #e5e5e5;
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
  background: #f5f5f5;
}

.dict-item.selected {
  background: #e3f2fd;
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
  color: #999;
}

.dict-detail {
  flex: 1;
  background: white;
  border: 1px solid #e5e5e5;
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
  background: #e9e9eb;
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
}

.tab.active {
  background: white;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.search-input {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 13px;
}

.sort-select {
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 13px;
}

.entries-table {
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  overflow: hidden;
}

.table-header {
  display: flex;
  padding: 8px 12px;
  background: #f5f5f5;
  font-size: 12px;
  font-weight: 600;
  color: #666;
}

.table-row {
  display: flex;
  padding: 6px 12px;
  font-size: 13px;
  border-top: 1px solid #f0f0f0;
}

.table-row:hover {
  background: #f9f9f9;
}

.col-idx {
  width: 50px;
  color: #999;
}

.col-word {
  flex: 1;
  font-weight: 500;
}

.col-code {
  flex: 1;
  font-family: monospace;
  color: #666;
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
  border: 1px solid #007aff;
  border-radius: 3px;
  font-size: 12px;
  text-align: right;
}

.icon-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  padding: 2px 4px;
}

.icon-btn.danger {
  color: #ff3b30;
}

.icon-btn.save {
  color: #34c759;
}

.pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 12px;
  font-size: 12px;
  color: #666;
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
  border: 1px solid #ddd;
  background: white;
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
  background: #f9f9f9;
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
  background: #e3f2fd;
  border-radius: 8px;
  color: #1976d2;
}

.snap-time {
  color: #999;
  font-size: 12px;
}

.snap-size {
  color: #999;
  font-size: 12px;
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

.batch-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #e5e5e5;
}

.btn.danger {
  color: #ff3b30;
  border-color: #ff3b30;
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
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.warning {
  color: #ff3b30;
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
  background: #007aff;
  color: white;
  border: none;
}

.btn-danger {
  background: #ff3b30;
  color: white;
  border: none;
}
</style>
