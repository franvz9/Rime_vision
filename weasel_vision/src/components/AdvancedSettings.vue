<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['changed'])

interface ConfigFileInfo {
  name: string
  path: string
  exists: boolean
  is_main: boolean
}

const userDir = ref('')
const configFiles = ref<ConfigFileInfo[]>([])
const showResetConfirm = ref(false)

onMounted(async () => {
  try {
    userDir.value = await invoke('get_rime_user_dir')
    configFiles.value = await invoke('get_config_files')
  } catch (e) {
    console.error('Failed to load settings:', e)
  }
})

async function sync() {
  try {
    await invoke('sync')
  } catch (e) {
    console.error('Sync failed:', e)
  }
}

async function resetConfig() {
  try {
    await invoke('reset_config')
    configFiles.value = await invoke('get_config_files')
    showResetConfirm.value = false
    emit('changed')
  } catch (e) {
    console.error('Reset failed:', e)
  }
}
</script>

<template>
  <div class="advanced-settings">
    <div class="section">
      <h3>Rime 目录</h3>
      <div class="dir-row">
        <span class="label">用户目录:</span>
        <code class="dir-path">{{ userDir }}</code>
      </div>
    </div>

    <div class="section">
      <h3>配置文件</h3>
      <div v-for="file in configFiles" :key="file.name" class="file-row">
        <span class="file-icon">{{ file.exists ? '📄' : '📃' }}</span>
        <div class="file-info">
          <div class="file-name">{{ file.name }}</div>
          <div class="file-desc">{{ file.is_main ? '主配置文件' : '自定义配置 (patch)' }}</div>
        </div>
        <span v-if="!file.exists" class="file-status">保存设置后自动生成</span>
      </div>
    </div>

    <div class="section">
      <h3>操作</h3>
      <div class="actions-row">
        <button class="btn btn-outline" @click="sync">同步用户数据</button>
        <button class="btn btn-danger" @click="showResetConfirm = true">重置自定义配置</button>
      </div>
    </div>

    <div class="section">
      <h3>关于</h3>
      <div class="about">
        <strong>WeaselVision</strong> <span class="version">v0.1.0</span>
        <p>Rime 输入法可视化配置工具（跨平台版）</p>
      </div>
    </div>

    <!-- Reset confirm -->
    <div v-if="showResetConfirm" class="modal-overlay" @click.self="showResetConfirm = false">
      <div class="modal">
        <h3>确认重置</h3>
        <p>将删除自定义配置文件，此操作不可撤销。</p>
        <div class="modal-actions">
          <button class="btn" @click="showResetConfirm = false">取消</button>
          <button class="btn btn-danger" @click="resetConfig">重置</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.advanced-settings {
  max-width: 500px;
}

.section {
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid #e5e5e5;
}

.section h3 {
  font-size: 15px;
  margin-bottom: 12px;
}

.dir-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dir-row .label {
  font-size: 14px;
}

.dir-path {
  font-size: 13px;
  background: #f5f5f5;
  padding: 4px 8px;
  border-radius: 4px;
}

.file-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 0;
}

.file-icon {
  font-size: 18px;
}

.file-info {
  flex: 1;
}

.file-name {
  font-size: 14px;
  font-family: monospace;
}

.file-desc {
  font-size: 12px;
  color: #999;
}

.file-status {
  font-size: 12px;
  color: #999;
}

.actions-row {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 8px 16px;
  border: 1px solid #ddd;
  background: white;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.btn-outline {
  background: white;
}

.btn-danger {
  background: #ff3b30;
  color: white;
  border: none;
}

.about .version {
  color: #999;
  font-size: 14px;
}

.about p {
  font-size: 13px;
  color: #666;
  margin-top: 4px;
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

.modal p {
  color: #666;
  font-size: 14px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}
</style>
