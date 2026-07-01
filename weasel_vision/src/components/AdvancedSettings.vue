<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../composables/useToast'
import { errorMessage } from '../utils'
import WeaselModal from './WeaselModal.vue'

const toast = useToast()


interface ConfigFileInfo {
  name: string
  path: string
  exists: boolean
  is_main: boolean
}

const userDir = ref('')
const configFiles = ref<ConfigFileInfo[]>([])
const showResetConfirm = ref(false)

const customFileNames = computed(() => {
  return configFiles.value
    .filter(f => !f.is_main)
    .map(f => f.name)
    .join('、') || '自定义配置文件'
})

let advMounted = true

onMounted(async () => {
  try {
    userDir.value = await invoke('get_rime_user_dir')
    if (!advMounted) return
    configFiles.value = await invoke('get_config_files')
    if (!advMounted) return
  } catch (e) {
    if (!advMounted) return
    toast.error(`加载设置失败: ${errorMessage(e)}`)
  }
})

onUnmounted(() => { advMounted = false })

async function resetConfig() {
  showResetConfirm.value = false
  try {
    await invoke('reset_config')
    configFiles.value = await invoke('get_config_files')
    toast.success('配置已重置')
  } catch (e) {
    toast.error(`重置配置失败: ${errorMessage(e)}`)
  }
}

async function openRimeDir() {
  try {
    await invoke('open_rime_dir')
  } catch (e) {
    toast.error(`打开 Rime 目录失败: ${errorMessage(e)}`)
  }
}
</script>

<template>
  <div class="advanced-settings">
    <div class="section">
      <h3>Rime 目录</h3>
      <div class="dir-row">
        <span class="label">用户目录:</span>
        <code class="dir-path clickable" @click="openRimeDir" title="点击打开此目录">{{ userDir }}</code>
        <button type="button" class="btn btn-small" @click="openRimeDir">📂 打开</button>
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
        <button class="btn btn-danger" @click="showResetConfirm = true">重置自定义配置</button>
      </div>
    </div>

    <div class="section">
      <h3>关于</h3>
      <div class="about">
        <strong>WeaselVision</strong> <span class="version">v0.2.0</span>
        <p>Rime 输入法可视化配置工具（跨平台版）</p>
      </div>
      <div class="donate-section">
        <p class="donate-text">制作不易，请作者喝杯咖啡吧 ☕</p>
        <img src="../assets/donate-qrcode.jpg" alt="捐赠二维码" class="donate-qrcode" />
      </div>
    </div>

    <!-- Reset confirm modal -->
    <WeaselModal :show="showResetConfirm" title="确认重置" @close="showResetConfirm = false">
      <p>将删除自定义配置文件（{{ customFileNames }}），此操作不可撤销。</p>
      <template #actions>
        <button class="btn" @click="showResetConfirm = false">取消</button>
        <button class="btn btn-danger" @click="resetConfig">确认重置</button>
      </template>
    </WeaselModal>
  </div>
</template>

<style scoped>
.advanced-settings {
  max-width: 500px;
}

.section {
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--color-border);
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
  background: var(--color-bg-tertiary);
  padding: 4px 8px;
  border-radius: 4px;
}

.dir-path.clickable {
  cursor: pointer;
  transition: background 0.2s;
}

.dir-path.clickable:hover {
  background: var(--color-bg-hover);
}

.btn-small {
  padding: 4px 10px;
  font-size: 12px;
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
  color: var(--color-text-tertiary);
}

.file-status {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.actions-row {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 8px 16px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.btn-danger {
  background: var(--color-danger);
  color: white;
  border: none;
}

.about .version {
  color: var(--color-text-tertiary);
  font-size: 14px;
}

.about p {
  font-size: 13px;
  color: var(--color-text-secondary);
  margin-top: 4px;
}

.donate-section {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--color-border);
}

.donate-text {
  font-size: 14px;
  color: var(--color-text-primary);
  margin-bottom: 12px;
}

.donate-qrcode {
  width: 200px;
  height: auto;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}


.modal-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-bg-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}

.modal {
  background: var(--color-bg-modal);
  border-radius: 12px;
  padding: 24px;
  width: 400px;
  box-shadow: var(--shadow-lg);
}

.modal h3 {
  margin-bottom: 8px;
}

.modal p {
  color: var(--color-text-secondary);
  font-size: 14px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}
</style>
