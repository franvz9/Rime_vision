<script setup lang="ts">
import { ref } from 'vue'
import ThemeEditor from './components/ThemeEditor.vue'
import SchemaManager from './components/SchemaManager.vue'
import GeneralSettings from './components/GeneralSettings.vue'
import KeybindingEditor from './components/KeybindingEditor.vue'
import PunctuationSettings from './components/PunctuationSettings.vue'
import GrammarModel from './components/GrammarModel.vue'
import AdvancedSettings from './components/AdvancedSettings.vue'
import { invoke } from '@tauri-apps/api/core'

const selectedTab = ref('general')
const isDeploying = ref(false)

const tabs = [
  { id: 'general', label: '通用设置', icon: '⚙' },
  { id: 'theme', label: '主题外观', icon: '🎨' },
  { id: 'schema', label: '方案管理', icon: '📝' },
  { id: 'grammar', label: '语言模型', icon: '🧠' },
  { id: 'keybinding', label: '快捷键', icon: '⌨' },
  { id: 'punctuation', label: '标点符号', icon: '🔤' },
  { id: 'advanced', label: '高级设置', icon: '🔧' },
]

async function deploy() {
  isDeploying.value = true
  try {
    await invoke('deploy')
  } catch (e) {
    console.error('Deploy failed:', e)
  }
  setTimeout(() => {
    isDeploying.value = false
  }, 1500)
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
        <AdvancedSettings v-else-if="selectedTab === 'advanced'" />
      </div>
    </main>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: #f5f5f7;
  color: #1d1d1f;
  overflow: hidden;
}

.app {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 200px;
  background: #ffffff;
  border-right: 1px solid #e5e5e5;
  display: flex;
  flex-direction: column;
}

.sidebar-title {
  padding: 16px;
  font-size: 16px;
  font-weight: 600;
  border-bottom: 1px solid #e5e5e5;
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
  color: #333;
  text-align: left;
}

.nav-item:hover {
  background: #f0f0f0;
}

.nav-item.active {
  background: #007aff;
  color: white;
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
  background: white;
  border-bottom: 1px solid #e5e5e5;
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
  background: #007aff;
  color: white;
}

.btn-deploy:hover {
  background: #0056b3;
}

.btn-deploy:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.content-body {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}
</style>
