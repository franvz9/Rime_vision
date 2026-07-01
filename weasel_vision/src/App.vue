<script setup lang="ts">
import { ref, onMounted, onUnmounted, defineAsyncComponent } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Lazy load heavy components for better initial load performance
const GeneralSettings = defineAsyncComponent(() => import('./components/GeneralSettings.vue'))
const ThemeEditor = defineAsyncComponent(() => import('./components/ThemeEditor.vue'))
const SchemaManager = defineAsyncComponent(() => import('./components/SchemaManager.vue'))
const GrammarModel = defineAsyncComponent(() => import('./components/GrammarModel.vue'))
const KeybindingEditor = defineAsyncComponent(() => import('./components/KeybindingEditor.vue'))
const PunctuationSettings = defineAsyncComponent(() => import('./components/PunctuationSettings.vue'))
const BackupManager = defineAsyncComponent(() => import('./components/BackupManager.vue'))
const DictManager = defineAsyncComponent(() => import('./components/DictManager.vue'))
const SyncManager = defineAsyncComponent(() => import('./components/SyncManager.vue'))
const AdvancedSettings = defineAsyncComponent(() => import('./components/AdvancedSettings.vue'))

interface PendingDelete {
  delete_type: string
  identifier: string
  label: string
}

const selectedTab = ref('general')
const isDeploying = ref(false)
const pendingDeletes = ref<PendingDelete[]>([])

// Event handler functions for proper cleanup
const handleNavigateToSync = () => {
  selectedTab.value = 'sync'
}

const handleAddPendingDelete = (e: Event) => {
  const { delete_type, identifier, label } = (e as CustomEvent).detail
  if (!pendingDeletes.value.find(d => d.delete_type === delete_type && d.identifier === identifier)) {
    pendingDeletes.value.push({ delete_type, identifier, label })
  }
}

const handleRemovePendingDelete = (e: Event) => {
  const { delete_type, identifier } = (e as CustomEvent).detail
  pendingDeletes.value = pendingDeletes.value.filter(
    d => !(d.delete_type === delete_type && d.identifier === identifier)
  )
}

// Register event listeners on mount, clean up on unmount
onMounted(() => {
  window.addEventListener('navigate-to-sync-settings', handleNavigateToSync)
  window.addEventListener('add-pending-delete', handleAddPendingDelete)
  window.addEventListener('remove-pending-delete', handleRemovePendingDelete)
})

onUnmounted(() => {
  window.removeEventListener('navigate-to-sync-settings', handleNavigateToSync)
  window.removeEventListener('add-pending-delete', handleAddPendingDelete)
  window.removeEventListener('remove-pending-delete', handleRemovePendingDelete)
})

const tabs = [
  { id: 'general', label: '通用设置', icon: '⚙️' },
  { id: 'theme', label: '主题外观', icon: '🎨' },
  { id: 'schema', label: '方案管理', icon: '📝' },
  { id: 'grammar', label: '语言模型', icon: '🧠' },
  { id: 'keybinding', label: '快捷键', icon: '⌨️' },
  { id: 'punctuation', label: '标点符号', icon: '🔤' },
  { id: 'backup', label: '备份管理', icon: '💾' },
  { id: 'dict', label: '词典管理', icon: '📖' },
  { id: 'sync', label: '同步管理', icon: '🔄' },
  { id: 'advanced', label: '高级设置', icon: '🔧' },
]

async function deploy() {
  isDeploying.value = true
  try {
    const deletes = pendingDeletes.value.map(d => ({
      delete_type: d.delete_type,
      identifier: d.identifier
    }))
    await invoke('deploy', { pendingDeletes: deletes.length > 0 ? deletes : null })
    // Clear pending deletes after successful deploy
    pendingDeletes.value = []
    // Notify all components that deploy completed
    window.dispatchEvent(new CustomEvent('deploy-complete'))
  } catch (e) {
    console.error('Deploy failed:', e)
  } finally {
    isDeploying.value = false
  }
}


</script>

<template>
  <div class="app">
    <aside class="sidebar">
      <div class="sidebar-title">RimeVision</div>
      <nav class="sidebar-nav">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="['nav-item', { active: selectedTab === tab.id }]"
          @click="selectedTab = tab.id"
        >
          <span class="nav-icon">{{ tab.icon }}</span>
          <span class="nav-label">{{ tab.label }}</span>
        </button>
      </nav>
    </aside>

    <main class="content">
      <header class="toolbar">
        <div class="toolbar-left">
          <h2 class="page-title">{{ tabs.find(t => t.id === selectedTab)?.label }}</h2>
        </div>
        <div class="toolbar-right">
          <span v-if="pendingDeletes.length > 0" class="delete-hint">
            🗑️ {{ pendingDeletes.length }} 项待删除：{{ pendingDeletes.map(d => d.label).join('、') }}
          </span>
          <button
            class="btn btn-deploy"
            :disabled="isDeploying"
            @click="deploy"
          >
            {{ isDeploying ? '部署中...' : '重新部署' }}
          </button>
        </div>
      </header>

      <div class="content-body">
        <GeneralSettings v-if="selectedTab === 'general'" />
        <ThemeEditor v-else-if="selectedTab === 'theme'" />
        <SchemaManager v-else-if="selectedTab === 'schema'" />
        <GrammarModel v-else-if="selectedTab === 'grammar'" />
        <KeybindingEditor v-else-if="selectedTab === 'keybinding'" />
        <PunctuationSettings v-else-if="selectedTab === 'punctuation'" />
        <BackupManager v-else-if="selectedTab === 'backup'" />
        <DictManager v-else-if="selectedTab === 'dict'" />
        <SyncManager v-else-if="selectedTab === 'sync'" />
        <AdvancedSettings v-else-if="selectedTab === 'advanced'" />
      </div>
    </main>
  </div>
</template>

<style>
:root {
  /* ========== 亮色模式（默认）========== */
  
  /* 背景与表面色 */
  --color-bg-primary: #f5f5f7;
  --color-bg-secondary: #ffffff;
  --color-bg-tertiary: #fafafa;
  --color-bg-hover: #f0f0f0;
  --color-bg-active: #e9e9eb;
  --color-bg-input: #ffffff;
  --color-bg-modal: #ffffff;
  --color-bg-overlay: rgba(0, 0, 0, 0.5);
  
  /* 文字颜色 */
  --color-text-primary: #1d1d1f;
  --color-text-secondary: #6e6e73;
  --color-text-tertiary: #86868b;
  --color-text-placeholder: #999999;
  --color-text-inverse: #ffffff;
  
  /* 边框与分割线 */
  --color-border: #e5e5e5;
  --color-border-light: #f0f0f0;
  --color-border-dark: #d0d0d0;
  
  /* 主题色 */
  --color-accent: #007aff;
  --color-accent-hover: #0051d5;
  --color-accent-light: #e3f2fd;
  --color-accent-muted: rgba(0, 122, 255, 0.1);
  
  /* 状态色 */
  --color-danger: #ff3b30;
  --color-danger-hover: #cc2f26;
  --color-danger-light: rgba(255, 59, 48, 0.1);
  --color-success: #34c759;
  --color-success-muted: rgba(52, 199, 89, 0.1);
  --color-warning: #ff9500;
  --color-warning-bg: #fff3cd;
  --color-warning-text: #856404;
  --color-warning-light: #fff3cd;
  --color-warning-dark: #856404;
  --color-warning-muted: rgba(255, 149, 0, 0.1);
  --color-pending: #ff9800;
  --color-pending-bg: #fff3e0;
  
  /* 按钮与交互 */
  --color-btn-default-bg: #ffffff;
  --color-btn-default-border: #ddd;
  --color-btn-default-hover: #f5f5f5;
  --color-btn-primary-bg: #007aff;
  --color-btn-primary-hover: #0056b3;
  --color-btn-danger-bg: #ff3b30;
  --color-btn-danger-hover: #cc2f26;
  
  /* 阴影 */
  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.1);
  --shadow-md: 0 2px 8px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 4px 16px rgba(0, 0, 0, 0.1);
  
  /* Spacing */
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 16px;
  --spacing-lg: 24px;
  
  /* Border radius */
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
}

/* ========== 暗色模式（跟随系统）========== */
@media (prefers-color-scheme: dark) {
  :root {
    /* 背景与表面色 */
    --color-bg-primary: #1c1c1e;
    --color-bg-secondary: #2c2c2e;
    --color-bg-tertiary: #3a3a3c;
    --color-bg-hover: #3a3a3c;
    --color-bg-active: #48484a;
    --color-bg-input: #1c1c1e;
    --color-bg-modal: #2c2c2e;
    --color-bg-overlay: rgba(0, 0, 0, 0.7);
    
    /* 文字颜色 */
    --color-text-primary: #ffffff;
    --color-text-secondary: #a0a0a0;
    --color-text-tertiary: #8e8e93;
    --color-text-placeholder: #666666;
    --color-text-inverse: #ffffff;
    
    /* 边框与分割线 */
    --color-border: #38383a;
    --color-border-light: #2c2c2e;
    --color-border-dark: #48484a;
    
    /* 主题色 */
    --color-accent: #0a84ff;
    --color-accent-hover: #007aff;
    --color-accent-light: rgba(10, 132, 255, 0.2);
    --color-accent-muted: rgba(10, 132, 255, 0.15);
    
    /* 状态色 */
    --color-danger: #ff453a;
    --color-danger-hover: #ff3b30;
    --color-danger-light: rgba(255, 69, 58, 0.15);
    --color-success: #30d158;
    --color-success-muted: rgba(48, 209, 88, 0.15);
    --color-warning: #ffd60a;
    --color-warning-bg: rgba(255, 214, 10, 0.15);
    --color-warning-text: #ffd60a;
    --color-warning-light: rgba(255, 214, 10, 0.15);
    --color-warning-dark: #ffd60a;
    --color-warning-muted: rgba(255, 214, 10, 0.1);
    --color-pending: #ff9f0a;
    --color-pending-bg: rgba(255, 159, 10, 0.15);
    
    /* 按钮与交互 */
    --color-btn-default-bg: #2c2c2e;
    --color-btn-default-border: #38383a;
    --color-btn-default-hover: #3a3a3c;
    --color-btn-primary-bg: #0a84ff;
    --color-btn-primary-hover: #007aff;
    --color-btn-danger-bg: #ff453a;
    --color-btn-danger-hover: #ff3b30;
    
    /* 阴影 */
    --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.3);
    --shadow-md: 0 2px 8px rgba(0, 0, 0, 0.3);
    --shadow-lg: 0 4px 16px rgba(0, 0, 0, 0.3);
  }
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  overflow: hidden;
}

.app {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 200px;
  background: var(--color-bg-secondary);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
}

.sidebar-title {
  padding: 16px;
  font-size: 16px;
  font-weight: 600;
  border-bottom: 1px solid var(--color-border);
}

.sidebar-nav {
  padding: 8px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: none;
  cursor: pointer;
  border-radius: 6px;
  font-size: 14px;
  color: var(--color-text-primary);
  text-align: left;
}

.nav-item:hover {
  background: var(--color-bg-hover);
}

.nav-item.active {
  background: var(--color-accent);
  color: var(--color-text-inverse);
}

.nav-icon {
  font-size: 16px;
}

.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
}

.page-title {
  font-size: 18px;
  font-weight: 600;
}

.btn {
  padding: 6px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-deploy {
  background: var(--color-btn-primary-bg);
  color: var(--color-text-inverse);
}

.btn-deploy:hover {
  background: var(--color-btn-primary-hover);
}

.btn-deploy:disabled {
  background: var(--color-border-dark);
  cursor: not-allowed;
}

.delete-hint {
  color: var(--color-danger);
  font-size: 13px;
  margin-right: 12px;
  white-space: nowrap;
}

.content-body {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}
</style>
