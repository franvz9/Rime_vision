<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'

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
    const [settingsResult, statusResult, devicesResult] = await Promise.all([
      invoke<SyncSettings>('get_sync_settings'),
      invoke<SyncStatus | null>('get_sync_status'),
      invoke<SyncedDevice[]>('list_synced_devices')
    ])
    settings.value = settingsResult
    status.value = statusResult
    devices.value = devicesResult
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

async function selectFolder() {
  const selected = await dialogOpen({
    directory: true,
    multiple: false,
    title: '选择同步目录'
  })
  if (selected && typeof selected === 'string') {
    editingDir.value = selected
  }
}

async function openSyncDir() {
  if (!settings.value.sync_dir) return
  try {
    await invoke('open_dir', { path: settings.value.sync_dir })
  } catch (e) {
    console.error('Failed to open sync directory:', e)
    alert('打开目录失败：' + String(e))
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
        <span class="value mono clickable" @click="openSyncDir" v-if="settings.sync_dir" title="点击打开此目录">{{ settings.sync_dir }}</span>
        <span class="value mono" v-else>未配置</span>
        <button v-if="settings.sync_dir" type="button" class="btn btn-small" @click="openSyncDir">📂 打开</button>
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
        
        <div class="info-box">
          <p><strong>ℹ️ 说明：</strong></p>
          <ul>
            <li>Rime 的同步功能<strong>没有自带同步服务</strong>，只是将用户内容读写到指定文件夹</li>
            <li>这个文件夹配合 <strong>iCloud、WebDAV、坚果云</strong>等文件同步服务可实现多设备同步</li>
            <li>如果不配置同步服务，只是读写到这个指定文件夹而已</li>
          </ul>
          <p class="small">同步内容包括：</p>
          <ul class="small">
            <li>用户自定义配置（*.custom.yaml，如 default.custom.yaml 等）</li>
            <li>用户词典快照（*.userdb.txt，即用户词库的文本导出）</li>
          </ul>
          <p class="small">注意：同步不会备份方案主配置文件（如 rime_mint.schema.yaml）和系统词典文件。</p>
        </div>
        
        <div class="form-group">
          <label>同步目录</label>
          <div style="display: flex; gap: 8px;">
            <input v-model="editingDir" placeholder="/path/to/sync/folder" class="input" style="flex: 1;" />
            <button type="button" class="btn btn-secondary" @click="selectFolder">📁 选择文件夹</button>
          </div>
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
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
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
  color: var(--color-text-secondary);
  min-width: 80px;
}

.mono {
  font-family: monospace;
}

.status-badge {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 10px;
  background: var(--color-danger);
  color: white;
}

.status-badge.ok {
  background: var(--color-success);
}

.status-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.result-card {
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
}

.result-card.success {
  border-color: var(--color-success);
}

.result-card.error {
  border-color: var(--color-danger);
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
  color: var(--color-text-secondary);
  margin-right: 8px;
}

.result-item {
  display: inline-block;
  margin-right: 8px;
  font-family: monospace;
  font-size: 12px;
  background: var(--color-bg-tertiary);
  padding: 1px 6px;
  border-radius: 3px;
}

.result-section.errors .result-item {
  background: var(--color-danger-light);
  color: var(--color-danger-hover);
}

.devices-section {
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
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
  background: var(--color-bg-tertiary);
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
  color: var(--color-text-tertiary);
}

.empty-state {
  text-align: center;
  color: var(--color-text-tertiary);
  padding: 20px;
  font-size: 13px;
}

.btn {
  padding: 6px 14px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.btn-primary {
  background: var(--color-accent);
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
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 13px;
  font-family: monospace;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
}

.hint {
  font-size: 11px;
  color: var(--color-text-tertiary);
  margin-top: 4px;
}

.info-box {
  background: var(--color-accent-muted);
  border-left: 3px solid var(--color-accent);
  padding: 12px 16px;
  margin-bottom: 16px;
  border-radius: 4px;
}

.info-box p {
  margin: 8px 0;
  font-size: 13px;
  line-height: 1.5;
}

.info-box ul {
  margin: 8px 0;
  padding-left: 20px;
}

.info-box li {
  margin: 4px 0;
  font-size: 13px;
  line-height: 1.5;
}

.info-box .small {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.status-row .mono.clickable:hover {
  color: var(--color-accent);
  cursor: pointer;
  text-decoration: underline;
}

.btn-small {
  padding: 4px 12px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
}
</style>
