<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface SyncSettings {
  sync_dir: string | null
  installation_id: string
  sync_user_dict: boolean
  sync_config: boolean
}

interface SyncStatus {
  configured: boolean
  last_sync_time: string | null
  sync_dir_exists: boolean
  current_id: string
}

interface SyncedDevice {
  id: string
  last_sync: string
  synced_dicts: string[]
  synced_configs: string[]
}

interface SyncResult {
  success: boolean
  uploaded: string[]
  downloaded: string[]
  errors: string[]
}

const settings = ref<SyncSettings>({ sync_dir: null, installation_id: '', sync_user_dict: true, sync_config: true })
const status = ref<SyncStatus | null>(null)
const devices = ref<SyncedDevice[]>([])
const isSyncing = ref(false)
const syncResult = ref<SyncResult | null>(null)
const showSettings = ref(false)
const editingDir = ref('')

onMounted(async () => {
  await loadData()
})

async function loadData() {
  try {
    settings.value = await invoke('get_sync_settings')
    status.value = await invoke('get_sync_status')
    devices.value = await invoke('list_synced_devices')
  } catch (e) {
    console.error('Failed to load sync data:', e)
  }
}

async function executeSync() {
  isSyncing.value = true
  syncResult.value = null
  try {
    syncResult.value = await invoke('execute_sync')
    await loadData()
  } catch (e) {
    console.error('Sync failed:', e)
    syncResult.value = { success: false, uploaded: [], downloaded: [], errors: [String(e)] }
  } finally {
    isSyncing.value = false
  }
}

function openSettings() {
  editingDir.value = settings.value.sync_dir || ''
  showSettings.value = true
}

async function saveSettings() {
  settings.value.sync_dir = editingDir.value || null
  try {
    await invoke('save_sync_settings', { settings: settings.value })
    showSettings.value = false
    await loadData()
  } catch (e) {
    console.error('Failed to save sync settings:', e)
  }
}
</script>

<template>
  <div class="sync-manager">
    <!-- Status card -->
    <div class="status-card">
      <div class="status-row">
        <span class="label">设备标识:</span>
        <span class="value mono">{{ status?.current_id || '-' }}</span>
      </div>
      <div class="status-row">
        <span class="label">同步目录:</span>
        <span class="value mono">{{ settings.sync_dir || '未配置' }}</span>
      </div>
      <div class="status-row">
        <span class="label">上次同步:</span>
        <span class="value">{{ status?.last_sync_time || '从未同步' }}</span>
      </div>
      <div class="status-row">
        <span class="label">状态:</span>
        <span :class="['status-badge', { ok: status?.configured && status?.sync_dir_exists }]">
          {{ status?.configured && status?.sync_dir_exists ? '已配置' : '未就绪' }}
        </span>
      </div>

      <div class="status-actions">
        <button class="btn btn-primary" @click="executeSync" :disabled="isSyncing || !status?.configured">
          {{ isSyncing ? '同步中...' : '立即同步' }}
        </button>
        <button class="btn" @click="openSettings">修改设置</button>
      </div>
    </div>

    <!-- Sync result -->
    <div v-if="syncResult" :class="['result-card', { success: syncResult.success, error: !syncResult.success }]">
      <h4>{{ syncResult.success ? '同步完成' : '同步部分完成' }}</h4>
      <div v-if="syncResult.uploaded.length" class="result-section">
        <span class="result-label">上传:</span>
        <span v-for="f in syncResult.uploaded" :key="f" class="result-item">{{ f }}</span>
      </div>
      <div v-if="syncResult.downloaded.length" class="result-section">
        <span class="result-label">下载:</span>
        <span v-for="f in syncResult.downloaded" :key="f" class="result-item">{{ f }}</span>
      </div>
      <div v-if="syncResult.errors.length" class="result-section errors">
        <span class="result-label">错误:</span>
        <span v-for="e in syncResult.errors" :key="e" class="result-item">{{ e }}</span>
      </div>
    </div>

    <!-- Synced devices -->
    <div class="devices-section">
      <h3>已同步的设备</h3>
      <div v-if="devices.length === 0" class="empty-state">
        <p>暂无其他设备</p>
      </div>
      <div v-else class="device-list">
        <div v-for="device in devices" :key="device.id" class="device-item">
          <div class="device-icon">💻</div>
          <div class="device-info">
            <div class="device-id">{{ device.id }}</div>
            <div class="device-meta">
              上次同步: {{ device.last_sync }}
              <span v-if="device.synced_dicts.length"> · 词典: {{ device.synced_dicts.join(', ') }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Settings modal -->
    <div v-if="showSettings" class="modal-overlay" @click.self="showSettings = false">
      <div class="modal">
        <h3>同步设置</h3>
        <div class="form-group">
          <label>同步目录</label>
          <input v-model="editingDir" placeholder="/path/to/sync/folder" class="input" />
          <p class="hint">例如: /Users/fred/Dropbox/RimeSync 或 D:\Dropbox\RimeSync</p>
        </div>
        <div class="form-group">
          <label>设备标识</label>
          <input v-model="settings.installation_id" class="input" />
          <p class="hint">建议使用小写字母、数字、横线和下划线</p>
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showSettings = false">取消</button>
          <button class="btn btn-primary" @click="saveSettings">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sync-manager {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 600px;
}

.status-card {
  background: white;
  border: 1px solid #e5e5e5;
  border-radius: 8px;
  padding: 16px;
}

.status-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  font-size: 13px;
}

.status-row .label {
  color: #666;
  min-width: 80px;
}

.mono {
  font-family: monospace;
}

.status-badge {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 10px;
  background: #ff3b30;
  color: white;
}

.status-badge.ok {
  background: #34c759;
}

.status-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.result-card {
  background: white;
  border: 1px solid #e5e5e5;
  border-radius: 8px;
  padding: 16px;
}

.result-card.success {
  border-color: #34c759;
}

.result-card.error {
  border-color: #ff3b30;
}

.result-card h4 {
  margin-bottom: 8px;
  font-size: 14px;
}

.result-section {
  margin-bottom: 6px;
  font-size: 13px;
}

.result-label {
  color: #666;
  margin-right: 8px;
}

.result-item {
  display: inline-block;
  margin-right: 8px;
  font-family: monospace;
  font-size: 12px;
  background: #f5f5f5;
  padding: 1px 6px;
  border-radius: 3px;
}

.result-section.errors .result-item {
  background: #ffebee;
  color: #c62828;
}

.devices-section {
  background: white;
  border: 1px solid #e5e5e5;
  border-radius: 8px;
  padding: 16px;
}

.devices-section h3 {
  font-size: 14px;
  margin-bottom: 12px;
}

.device-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.device-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  border-radius: 6px;
  background: #f9f9f9;
}

.device-icon {
  font-size: 20px;
}

.device-info {
  flex: 1;
}

.device-id {
  font-size: 13px;
  font-family: monospace;
}

.device-meta {
  font-size: 11px;
  color: #999;
}

.empty-state {
  text-align: center;
  color: #999;
  padding: 20px;
  font-size: 13px;
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
  width: 450px;
}

.modal h3 {
  margin-bottom: 16px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 4px;
}

.input {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 13px;
  font-family: monospace;
}

.hint {
  font-size: 11px;
  color: #999;
  margin-top: 4px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
