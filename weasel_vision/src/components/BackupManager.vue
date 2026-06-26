<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface BackupInfo {
  id: string
  created_at: string
  backup_type: string
  file_count: number
  total_size: number
  note: string | null
}

interface BackupFile {
  name: string
  size: number
  modified: string
}

interface BackupDetail {
  info: BackupInfo
  files: BackupFile[]
}

interface FileDiff {
  file_name: string
  current: string | null
  backup: string
}

const backups = ref<BackupInfo[]>([])
const selectedBackup = ref<BackupDetail | null>(null)
const showRestoreDialog = ref(false)
const restoreFiles = ref<string[]>([])
const showDeleteConfirm = ref(false)
const backupToDelete = ref<string | null>(null)
const showCompareDialog = ref(false)
const compareDiff = ref<FileDiff | null>(null)
const createNote = ref('')
const isCreating = ref(false)
const isRestoring = ref(false)

onMounted(async () => {
  await loadBackups()
})

async function loadBackups() {
  try {
    backups.value = await invoke('list_backups')
  } catch (e) {
    console.error('Failed to load backups:', e)
  }
}

async function selectBackup(backup: BackupInfo) {
  try {
    const detail = await invoke('get_backup_detail', { backupId: backup.id })
    selectedBackup.value = detail as BackupDetail
    restoreFiles.value = selectedBackup.value.files.map(f => f.name)
  } catch (e) {
    console.error('Failed to load backup detail:', e)
  }
}

async function createBackup() {
  isCreating.value = true
  try {
    await invoke('create_backup', { note: createNote.value || null })
    createNote.value = ''
    await loadBackups()
  } catch (e) {
    console.error('Failed to create backup:', e)
  } finally {
    isCreating.value = false
  }
}

async function restoreBackup() {
  if (!selectedBackup.value) return
  isRestoring.value = true
  try {
    await invoke('restore_backup', {
      backupId: selectedBackup.value.info.id,
      restoreFiles: restoreFiles.value,
    })
    showRestoreDialog.value = false
  } catch (e) {
    console.error('Failed to restore backup:', e)
  } finally {
    isRestoring.value = false
  }
}

async function compareBackup(fileName: string) {
  if (!selectedBackup.value) return
  try {
    compareDiff.value = await invoke('compare_backup', {
      backupId: selectedBackup.value.info.id,
      fileName,
    })
    showCompareDialog.value = true
  } catch (e) {
    console.error('Failed to compare backup:', e)
  }
}

async function deleteBackup(id: string) {
  backupToDelete.value = id
  showDeleteConfirm.value = true
}

async function confirmDelete() {
  if (!backupToDelete.value) return
  try {
    await invoke('delete_backup', { backupId: backupToDelete.value })
    if (selectedBackup.value?.info.id === backupToDelete.value) {
      selectedBackup.value = null
    }
    await loadBackups()
  } catch (e) {
    console.error('Failed to delete backup:', e)
  } finally {
    showDeleteConfirm.value = false
    backupToDelete.value = null
  }
}

function formatSize(bytes: number): string {
  if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${bytes} B`
}

function typeName(type: string): string {
  const map: Record<string, string> = { manual: '手动', deploy: '部署前', auto: '自动' }
  return map[type] || type
}
</script>

<template>
  <div class="backup-manager">
    <div class="layout">
      <div class="backup-list">
        <div class="list-header">
          <h3>备份列表</h3>
          <button class="btn btn-primary btn-sm" @click="createBackup" :disabled="isCreating">
            {{ isCreating ? '创建中...' : '+ 创建备份' }}
          </button>
        </div>

        <div v-if="backups.length === 0" class="empty-state">
          <p>暂无备份</p>
          <p class="hint">点击「创建备份」保存当前所有配置</p>
        </div>

        <div v-else class="backup-items">
          <div
            v-for="backup in backups"
            :key="backup.id"
            :class="['backup-item', { selected: selectedBackup?.info.id === backup.id }]"
            @click="selectBackup(backup)"
          >
            <div class="backup-icon">📁</div>
            <div class="backup-info">
              <div class="backup-id">{{ backup.id }}</div>
              <div class="backup-meta">
                {{ typeName(backup.backup_type) }} · {{ backup.file_count }} 文件 · {{ formatSize(backup.total_size) }}
              </div>
            </div>
            <button class="icon-btn danger" @click.stop="deleteBackup(backup.id)">🗑</button>
          </div>
        </div>
      </div>

      <div class="backup-detail">
        <template v-if="selectedBackup">
          <div class="detail-header">
            <h3>备份详情</h3>
            <span class="backup-type-badge">{{ typeName(selectedBackup.info.backup_type) }}</span>
          </div>

          <div class="detail-meta">
            <div><span class="label">时间:</span> {{ selectedBackup.info.created_at }}</div>
            <div><span class="label">文件数:</span> {{ selectedBackup.info.file_count }}</div>
            <div><span class="label">大小:</span> {{ formatSize(selectedBackup.info.total_size) }}</div>
            <div v-if="selectedBackup.info.note"><span class="label">备注:</span> {{ selectedBackup.info.note }}</div>
          </div>

          <h4>包含文件</h4>
          <div class="file-list">
            <div v-for="file in selectedBackup.files" :key="file.name" class="file-item">
              <span class="file-icon">📄</span>
              <span class="file-name">{{ file.name }}</span>
              <span class="file-size">{{ formatSize(file.size) }}</span>
              <button class="icon-btn" @click="compareBackup(file.name)">对比</button>
            </div>
          </div>

          <div class="detail-actions">
            <button class="btn btn-primary" @click="showRestoreDialog = true">恢复此备份</button>
          </div>
        </template>
        <div v-else class="empty-detail">
          <p>选择一个备份查看详情</p>
        </div>
      </div>
    </div>

    <!-- Restore dialog -->
    <div v-if="showRestoreDialog" class="modal-overlay" @click.self="showRestoreDialog = false">
      <div class="modal">
        <h3>恢复备份 — {{ selectedBackup?.info.id }}</h3>
        <p class="hint">选择要恢复的文件，当前文件会被备份到 backups/auto/</p>
        <div class="restore-files">
          <label v-for="file in selectedBackup?.files" :key="file.name" class="checkbox">
            <input type="checkbox" :value="file.name" v-model="restoreFiles" />
            {{ file.name }}
          </label>
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showRestoreDialog = false">取消</button>
          <button class="btn btn-primary" @click="restoreBackup" :disabled="restoreFiles.length === 0 || isRestoring">
            {{ isRestoring ? '恢复中...' : '确认恢复' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Compare dialog -->
    <div v-if="showCompareDialog && compareDiff" class="modal-overlay" @click.self="showCompareDialog = false">
      <div class="modal modal-wide">
        <h3>对比: {{ compareDiff.file_name }}</h3>
        <div class="diff-view">
          <div v-if="compareDiff.current === null" class="diff-note">当前文件不存在</div>
          <pre v-else class="diff-current">{{ compareDiff.current }}</pre>
          <pre class="diff-backup">{{ compareDiff.backup }}</pre>
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showCompareDialog = false">关闭</button>
        </div>
      </div>
    </div>

    <!-- Delete confirm dialog -->
    <div v-if="showDeleteConfirm" class="modal-overlay" @click.self="showDeleteConfirm = false">
      <div class="modal">
        <h3>确认删除</h3>
        <p>确定要删除备份 <strong>{{ backupToDelete }}</strong> 吗？此操作不可撤销。</p>
        <div class="modal-actions">
          <button class="btn" @click="showDeleteConfirm = false">取消</button>
          <button class="btn btn-danger" @click="confirmDelete">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backup-manager {
  display: flex;
  flex-direction: column;
}

.layout {
  display: flex;
  gap: 16px;
  min-height: 400px;
}

.backup-list {
  width: 320px;
  background: white;
  border: 1px solid #e5e5e5;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #e5e5e5;
}

.list-header h3 {
  font-size: 14px;
  margin: 0;
}

.backup-items {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.backup-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  border-radius: 6px;
  cursor: pointer;
}

.backup-item:hover {
  background: #f5f5f5;
}

.backup-item.selected {
  background: #e3f2fd;
}

.backup-icon {
  font-size: 20px;
}

.backup-info {
  flex: 1;
  min-width: 0;
}

.backup-id {
  font-size: 13px;
  font-family: monospace;
}

.backup-meta {
  font-size: 11px;
  color: #999;
}

.backup-detail {
  flex: 1;
  background: white;
  border: 1px solid #e5e5e5;
  border-radius: 8px;
  padding: 16px;
}

.detail-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.detail-header h3 {
  font-size: 15px;
  margin: 0;
}

.backup-type-badge {
  font-size: 11px;
  padding: 2px 8px;
  background: #e3f2fd;
  color: #1976d2;
  border-radius: 10px;
}

.detail-meta {
  font-size: 13px;
  margin-bottom: 16px;
}

.detail-meta .label {
  color: #666;
  margin-right: 8px;
}

.detail-meta div {
  margin-bottom: 4px;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 16px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  font-size: 13px;
  background: #f9f9f9;
  border-radius: 4px;
}

.file-icon {
  font-size: 14px;
}

.file-name {
  flex: 1;
  font-family: monospace;
}

.file-size {
  color: #999;
  font-size: 12px;
}

.detail-actions {
  margin-top: 16px;
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

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
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

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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
  max-height: 80vh;
  overflow-y: auto;
}

.modal-wide {
  width: 700px;
}

.modal h3 {
  margin-bottom: 8px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}

.restore-files {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 12px 0;
}

.checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-family: monospace;
  cursor: pointer;
}

.diff-view {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin: 12px 0;
}

.diff-view pre {
  background: #f5f5f5;
  padding: 8px;
  border-radius: 4px;
  font-size: 11px;
  overflow-x: auto;
  max-height: 300px;
  overflow-y: auto;
}

.diff-current {
  border: 1px solid #ddd;
}

.diff-backup {
  border: 1px solid #007aff;
  background: #e3f2fd;
}

.diff-note {
  grid-column: 1 / -1;
  color: #999;
  font-size: 13px;
  text-align: center;
  padding: 20px;
}
</style>
