<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import DeployNotice from './DeployNotice.vue'
import WeaselModal from './WeaselModal.vue'
import { useToast } from '../composables/useToast'
import {
  hexToRgb,
  rgbToHex,
  hexToRimeHex,
  errorMessage,
  emitBusEvent,
  BusEvents,
  type RgbColor,
} from '../utils'

interface ColorScheme {
  name: string
  author: string
  color_space: string
  back_color: RgbColor | null
  border_color: RgbColor | null
  text_color: RgbColor | null
  hilited_text_color: RgbColor | null
  hilited_back_color: RgbColor | null
  hilited_candidate_back_color: RgbColor | null
  candidate_text_color: RgbColor | null
  hilited_candidate_text_color: RgbColor | null
  candidate_label_color: RgbColor | null
  hilited_candidate_label_color: RgbColor | null
  comment_text_color: RgbColor | null
  hilited_comment_text_color: RgbColor | null
  preedit_back_color: RgbColor | null
  candidate_back_color: RgbColor | null
}

interface StyleSettings {
  font_face?: string
  font_point?: number
  label_font_face?: string
  label_font_point?: number
  comment_font_face?: string
  comment_font_point?: number
  candidate_list_layout?: string
  text_orientation?: string
  candidate_format?: string
  status_message_type?: string
  corner_radius?: number
  hilited_corner_radius?: number
  line_spacing?: number
  spacing?: number
  border_height?: number
  border_width?: number
  shadow_size?: number
  alpha?: number
  inline_preedit?: boolean
  inline_candidate?: boolean
  translucency?: boolean
  mutual_exclusive?: boolean
  memorize_size?: boolean
  show_paging?: boolean
  [key: string]: unknown
}

interface StyleData {
  style: StyleSettings
  light_schemes: Record<string, ColorScheme>
  dark_schemes: Record<string, ColorScheme>
  selected_light: string
  selected_dark: string
}

const toast = useToast()
const styleData = ref<StyleData | null>(null)
const activeTab = ref<'light' | 'dark' | 'style'>('light')
const selectedScheme = ref<string>('')
const editingScheme = ref<ColorScheme | null>(null)
const showEditor = ref(false)
const localStyle = ref<StyleSettings | null>(null)

// Pending scheme (applied but not yet deployed)
const pendingScheme = ref<string | null>(null)

// Edit mode: 'edit' (overwrite) or 'copy' (save as new)
const editMode = ref<'edit' | 'copy'>('edit')
const originalSchemeName = ref<string>('')
const newSchemeName = ref<string>('')

// Pending delete schemes (marked for deletion, will be deleted on deploy)
const pendingDeleteSchemes = ref<Set<string>>(new Set())

const schemes = computed(() => {
  if (!styleData.value) return {}
  return activeTab.value === 'light' ? styleData.value.light_schemes : styleData.value.dark_schemes
})

const currentActiveScheme = computed(() => {
  if (!styleData.value) return ''
  return activeTab.value === 'light'
    ? styleData.value.selected_light
    : styleData.value.selected_dark
})

// Update selectedScheme when tab changes to show the currently active scheme
watch(activeTab, (newTab) => {
  if (!styleData.value) return
  selectedScheme.value =
    newTab === 'light' ? styleData.value!.selected_light : styleData.value!.selected_dark
})

// Refresh style data after deploy completes
const handleDeployComplete = async () => {
  try {
    styleData.value = await invoke('get_style_data')
    localStyle.value = { ...styleData.value!.style }
    pendingScheme.value = null
  } catch (e) {
    toast.error(`刷新主题数据失败: ${errorMessage(e)}`)
  }
}

let themeMounted = true

onMounted(async () => {
  try {
    styleData.value = await invoke('get_style_data')
    if (!themeMounted) return
    selectedScheme.value =
      activeTab.value === 'light' ? styleData.value!.selected_light : styleData.value!.selected_dark
    localStyle.value = { ...styleData.value!.style }
  } catch (e) {
    if (!themeMounted) return
    toast.error(`加载主题数据失败: ${errorMessage(e)}`)
    return
  }
  if (themeMounted) {
    window.addEventListener(BusEvents.DEPLOY_COMPLETE, handleDeployComplete)
  }
})

onUnmounted(() => {
  themeMounted = false
  window.removeEventListener(BusEvents.DEPLOY_COMPLETE, handleDeployComplete)
})

function selectScheme(name: string) {
  // Just update the preview selection, don't apply yet
  selectedScheme.value = name
}

async function applyScheme(name: string) {
  // Don't apply if marked for deletion
  if (pendingDeleteSchemes.value.has(name)) return

  // First select this scheme
  selectedScheme.value = name

  // Only set pending if the scheme is different from current active
  if (name === currentActiveScheme.value) {
    // Clicking on already active scheme - clear pending
    pendingScheme.value = null
  } else if (pendingScheme.value === name) {
    // Cancel pending for this scheme
    pendingScheme.value = null
  } else {
    // Set as pending
    pendingScheme.value = name
    // Immediately save the selection to backend
    try {
      await invoke('save_selected_schemes', {
        light: activeTab.value === 'light' ? name : styleData.value?.selected_light || 'native',
        dark: activeTab.value === 'dark' ? name : styleData.value?.selected_dark || 'native',
      })
      // Update local styleData to reflect the new selection
      if (styleData.value) {
        if (activeTab.value === 'light') {
          styleData.value.selected_light = name
        } else {
          styleData.value.selected_dark = name
        }
      }
    } catch (e) {
      toast.error(`保存方案选择失败: ${errorMessage(e)}`)
    }
  }
}

function editScheme(name: string) {
  // Don't edit if marked for deletion
  if (pendingDeleteSchemes.value.has(name)) return

  // First select this scheme
  selectedScheme.value = name

  const scheme = schemes.value[name]
  if (scheme) {
    editingScheme.value = { ...scheme }
    editMode.value = 'edit'
    originalSchemeName.value = name
    newSchemeName.value = name
    showEditor.value = true
  }
}

function copyScheme(name: string) {
  // Don't copy if marked for deletion
  if (pendingDeleteSchemes.value.has(name)) return

  // First select this scheme
  selectedScheme.value = name

  const scheme = schemes.value[name]
  if (scheme) {
    editingScheme.value = { ...scheme }
    editMode.value = 'copy'
    originalSchemeName.value = name
    newSchemeName.value = name + '_copy'
    showEditor.value = true
  }
}

async function saveScheme() {
  if (!editingScheme.value) return
  try {
    const isCopy = editMode.value === 'copy'
    const targetName = isCopy ? newSchemeName.value : editingScheme.value.name
    const origName = isCopy
      ? null
      : originalSchemeName.value !== targetName
        ? originalSchemeName.value
        : null

    await invoke('save_color_scheme', {
      name: targetName,
      scheme: editingScheme.value,
      originalName: origName,
    })
    showEditor.value = false
    styleData.value = await invoke('get_style_data')

    // If we modified the currently active scheme, set it as pending
    if (!isCopy && originalSchemeName.value === currentActiveScheme.value) {
      pendingScheme.value = targetName
    }
  } catch (e) {
    toast.error(`保存配色方案失败: ${errorMessage(e)}`)
  }
}

function cancelEdit() {
  showEditor.value = false
  editingScheme.value = null
}

function canDeleteScheme(name: string): boolean {
  // Cannot delete the currently active theme
  if (currentActiveScheme.value === name) return false
  // Cannot delete a theme pending deploy
  if (pendingScheme.value === name) return false
  return true
}

function toggleDeleteScheme(name: string) {
  if (!canDeleteScheme(name)) return
  if (pendingDeleteSchemes.value.has(name)) {
    pendingDeleteSchemes.value.delete(name)
    emitBusEvent(BusEvents.REMOVE_PENDING_DELETE, { delete_type: 'theme', identifier: name })
  } else {
    pendingDeleteSchemes.value.add(name)
    emitBusEvent(BusEvents.ADD_PENDING_DELETE, {
      delete_type: 'theme',
      identifier: name,
      label: `主题: ${name}`,
    })
    // If this scheme was pending apply, cancel it
    if (pendingScheme.value === name) {
      pendingScheme.value = null
    }
  }
  pendingDeleteSchemes.value = new Set(pendingDeleteSchemes.value)
}

async function importColorScheme() {
  const selected = await dialogOpen({
    directory: false,
    multiple: false,
    title: '选择配色方案文件',
    filters: [{ name: 'YAML Files', extensions: ['yaml', 'yml'] }],
  })
  if (selected && typeof selected === 'string') {
    try {
      await invoke('import_color_scheme', { filePath: selected })
      styleData.value = await invoke('get_style_data')
      toast.success('配色方案导入成功')
    } catch (e) {
      toast.error(`导入配色方案失败: ${errorMessage(e)}`)
    }
  }
}

function parseColor(v: unknown): RgbColor | null {
  if (!v) return null
  if (typeof v === 'object' && v !== null && 'r' in v) return v as RgbColor
  if (typeof v === 'string') return hexToRgb(v)
  return null
}

function colorToRgba(v: unknown): string {
  const c = parseColor(v)
  if (!c) return 'transparent'
  return `rgba(${c.r}, ${c.g}, ${c.b}, ${c.a / 255})`
}

function getColorHex(v: unknown): string {
  const c = parseColor(v)
  if (!c) return '#000000'
  return rgbToHex(c)
}

function getHexDisplay(v: unknown): string {
  const c = parseColor(v)
  if (!c) return ''
  if (c.a < 255) {
    return `0x${c.a.toString(16).padStart(2, '0')}${c.b.toString(16).padStart(2, '0')}${c.g.toString(16).padStart(2, '0')}${c.r.toString(16).padStart(2, '0')}`
  }
  return `0x${c.b.toString(16).padStart(2, '0')}${c.g.toString(16).padStart(2, '0')}${c.r.toString(16).padStart(2, '0')}`
}

function setColorHex(scheme: ColorScheme, key: string, hex: string) {
  const rgb = hexToRgb(hexToRimeHex(hex))
  if (rgb) {
    ;(scheme as unknown as Record<string, unknown>)[key] = rgb
  }
}

async function saveStyle() {
  if (!localStyle.value) return
  try {
    await invoke('save_style', { newStyle: localStyle.value })
    styleData.value = await invoke('get_style_data')
    localStyle.value = { ...styleData.value!.style }
    toast.success('样式已保存')
  } catch (e) {
    toast.error(`保存样式失败: ${errorMessage(e)}`)
  }
}
</script>

<template>
  <div class="theme-editor">
    <DeployNotice />

    <div class="tabs">
      <button :class="['tab', { active: activeTab === 'light' }]" @click="activeTab = 'light'">
        亮色主题
      </button>
      <button :class="['tab', { active: activeTab === 'dark' }]" @click="activeTab = 'dark'">
        暗色主题
      </button>
      <button :class="['tab', { active: activeTab === 'style' }]" @click="activeTab = 'style'">
        样式
      </button>
    </div>

    <div v-if="activeTab !== 'style'" class="scheme-list">
      <div class="list-header">
        <h3>{{ activeTab === 'light' ? '亮色主题' : '暗色主题' }}</h3>
        <button class="wv-btn btn-sm" @click="importColorScheme">📥 导入主题</button>
      </div>
      <!-- Pending notice -->
      <div v-if="pendingScheme" class="pending-notice">
        <span class="notice-text">已选择「{{ pendingScheme }}」，请点击顶部「重新部署」生效</span>
      </div>
      <div
        v-for="(scheme, name) in schemes"
        :key="name"
        :class="[
          'scheme-item',
          {
            'is-active': currentActiveScheme === name && pendingScheme !== name,
            'is-pending': pendingScheme === name,
            'is-selected': selectedScheme === name,
            'is-deleting': pendingDeleteSchemes.has(name as string),
          },
        ]"
        @click="selectScheme(name as string)"
      >
        <div class="scheme-color" :style="{ background: colorToRgba(scheme.back_color) }"></div>
        <div class="scheme-info">
          <div class="scheme-name-row">
            <span class="scheme-name">{{ name }}</span>
            <span
              v-if="
                currentActiveScheme === name &&
                pendingScheme !== name &&
                !pendingDeleteSchemes.has(name as string)
              "
              class="badge badge-active"
              >使用中</span
            >
            <span v-if="pendingScheme === name" class="badge badge-pending">待生效</span>
            <span v-if="pendingDeleteSchemes.has(name as string)" class="badge badge-deleting"
              >待删除</span
            >
          </div>
          <div v-if="scheme.author" class="scheme-author">{{ scheme.author }}</div>
        </div>
        <button
          v-if="pendingDeleteSchemes.has(name as string)"
          class="action-btn btn-cancel-delete"
          @click.stop="toggleDeleteScheme(name as string)"
        >
          取消删除
        </button>
        <button
          v-else-if="pendingScheme === name"
          class="action-btn btn-cancel"
          @click.stop="applyScheme(name)"
        >
          取消应用
        </button>
        <button v-else class="action-btn btn-apply" @click.stop="applyScheme(name)">
          应用主题
        </button>
        <button class="edit-btn" @click.stop="editScheme(name as string)">编辑</button>
        <button class="edit-btn" @click.stop="copyScheme(name as string)">复制</button>
        <button
          class="edit-btn btn-delete"
          :disabled="!canDeleteScheme(name as string)"
          :title="
            !canDeleteScheme(name as string)
              ? currentActiveScheme === name
                ? '使用中不可删除'
                : '待生效不可删除'
              : '删除'
          "
          @click.stop="toggleDeleteScheme(name as string)"
        >
          {{ pendingDeleteSchemes.has(name as string) ? '✖' : '🗑' }}
        </button>
      </div>
    </div>

    <!-- Style editing tab -->
    <div v-if="activeTab === 'style' && localStyle" class="style-editor">
      <div class="style-section">
        <h4>字体</h4>
        <div class="style-grid">
          <div class="style-field">
            <label>主字体</label>
            <input v-model="localStyle.font_face" placeholder="PingFang SC" />
          </div>
          <div class="style-field">
            <label>字号</label>
            <input
              v-model.number="localStyle.font_point"
              type="number"
              step="0.5"
              min="8"
              max="72"
            />
          </div>
          <div class="style-field">
            <label>序号字体</label>
            <input v-model="localStyle.label_font_face" placeholder="Lucida Grande" />
          </div>
          <div class="style-field">
            <label>序号字号</label>
            <input
              v-model.number="localStyle.label_font_point"
              type="number"
              step="0.5"
              min="8"
              max="72"
            />
          </div>
          <div class="style-field">
            <label>注释字体</label>
            <input v-model="localStyle.comment_font_face" />
          </div>
          <div class="style-field">
            <label>注释字号</label>
            <input
              v-model.number="localStyle.comment_font_point"
              type="number"
              step="0.5"
              min="8"
              max="72"
            />
          </div>
        </div>
      </div>

      <div class="style-section">
        <h4>窗口样式</h4>
        <div class="style-grid">
          <div class="style-field">
            <label>候选排列</label>
            <select v-model="localStyle.candidate_list_layout">
              <option value="stacked">纵向</option>
              <option value="linear">横向</option>
            </select>
          </div>
          <div class="style-field">
            <label>文字方向</label>
            <select v-model="localStyle.text_orientation">
              <option value="horizontal">横排</option>
              <option value="vertical">竖排</option>
            </select>
          </div>
          <div class="style-field">
            <label>候选格式</label>
            <input
              v-model="localStyle.candidate_format"
              placeholder="[label]. [candidate] [comment]"
            />
          </div>
          <div class="style-field">
            <label>状态提示</label>
            <select v-model="localStyle.status_message_type">
              <option value="mix">混合</option>
              <option value="long">完整</option>
              <option value="short">简短</option>
              <option value="never">不显示</option>
            </select>
          </div>
        </div>
      </div>

      <div class="style-section">
        <h4>尺寸与间距</h4>
        <div class="style-grid">
          <div class="style-field">
            <label>圆角半径</label>
            <input
              v-model.number="localStyle.corner_radius"
              type="number"
              step="1"
              min="0"
              max="50"
            />
          </div>
          <div class="style-field">
            <label>高亮圆角</label>
            <input
              v-model.number="localStyle.hilited_corner_radius"
              type="number"
              step="1"
              min="0"
              max="50"
            />
          </div>
          <div class="style-field">
            <label>行间距</label>
            <input
              v-model.number="localStyle.line_spacing"
              type="number"
              step="1"
              min="0"
              max="50"
            />
          </div>
          <div class="style-field">
            <label>间距</label>
            <input v-model.number="localStyle.spacing" type="number" step="1" min="0" max="50" />
          </div>
          <div class="style-field">
            <label>边框高度</label>
            <input
              v-model.number="localStyle.border_height"
              type="number"
              step="1"
              min="-20"
              max="50"
            />
          </div>
          <div class="style-field">
            <label>边框宽度</label>
            <input
              v-model.number="localStyle.border_width"
              type="number"
              step="1"
              min="-20"
              max="50"
            />
          </div>
          <div class="style-field">
            <label>阴影大小</label>
            <input
              v-model.number="localStyle.shadow_size"
              type="number"
              step="1"
              min="0"
              max="30"
            />
          </div>
          <div class="style-field">
            <label>透明度</label>
            <input v-model.number="localStyle.alpha" type="number" step="0.05" min="0" max="1" />
          </div>
        </div>
      </div>

      <div class="style-section">
        <h4>行为选项</h4>
        <div class="style-toggles">
          <label class="toggle-field">
            <input v-model="localStyle.inline_preedit" type="checkbox" />
            <span>内嵌编辑</span>
          </label>
          <label class="toggle-field">
            <input v-model="localStyle.inline_candidate" type="checkbox" />
            <span>内嵌候选</span>
          </label>
          <label class="toggle-field">
            <input v-model="localStyle.translucency" type="checkbox" />
            <span>半透明</span>
          </label>
          <label class="toggle-field">
            <input v-model="localStyle.mutual_exclusive" type="checkbox" />
            <span>颜色互斥</span>
          </label>
          <label class="toggle-field">
            <input v-model="localStyle.memorize_size" type="checkbox" />
            <span>记忆宽度</span>
          </label>
          <label class="toggle-field">
            <input v-model="localStyle.show_paging" type="checkbox" />
            <span>显示翻页</span>
          </label>
        </div>
      </div>

      <div class="style-actions">
        <button class="wv-btn wv-btn-primary" @click="saveStyle">保存样式</button>
      </div>
    </div>

    <!-- Simple preview -->
    <div v-if="activeTab !== 'style'" class="preview">
      <h3>预览</h3>
      <div v-if="selectedScheme && schemes[selectedScheme]" class="candidate-window">
        <div
          class="candidate-bg"
          :style="{ background: colorToRgba(schemes[selectedScheme].back_color) }"
        >
          <div
            class="candidate-item"
            :style="{
              background: colorToRgba(schemes[selectedScheme].hilited_candidate_back_color),
              color: colorToRgba(schemes[selectedScheme].hilited_candidate_text_color),
            }"
          >
            <span class="label">1.</span>
            <span class="text">候选词</span>
          </div>
          <div
            class="candidate-item"
            :style="{ color: colorToRgba(schemes[selectedScheme].candidate_text_color) }"
          >
            <span
              class="label"
              :style="{ color: colorToRgba(schemes[selectedScheme].candidate_label_color) }"
              >2.</span
            >
            <span class="text">示例</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Edit / Copy modal -->
    <WeaselModal
      :show="showEditor && !!editingScheme"
      :title="
        editMode === 'copy'
          ? `复制配色: ${originalSchemeName} → ${newSchemeName}`
          : `编辑配色: ${editingScheme!.name}`
      "
      @close="cancelEdit"
    >
      <div v-if="editMode === 'copy'" class="wv-form-group">
        <label>新方案名称</label>
        <input v-model="newSchemeName" placeholder="输入新方案名称" />
      </div>
      <div class="wv-form-group">
        <label>作者</label>
        <input v-model="editingScheme!.author" />
      </div>
      <div class="color-grid">
        <div
          v-for="(label, key) in {
            back_color: '背景色',
            text_color: '文字色',
            candidate_text_color: '候选文字',
            hilited_candidate_back_color: '高亮背景',
            hilited_candidate_text_color: '高亮文字',
            candidate_label_color: '编号色',
            comment_text_color: '注释色',
          }"
          :key="key"
          class="color-field"
        >
          <label>{{ label }}</label>
          <div class="color-input">
            <input
              type="color"
              :value="getColorHex(editingScheme![key as keyof ColorScheme])"
              @input="setColorHex(editingScheme!, key, ($event.target as HTMLInputElement).value)"
            />
            <span>{{ getHexDisplay(editingScheme![key as keyof ColorScheme]) }}</span>
          </div>
        </div>
      </div>
      <template #actions>
        <button class="wv-btn" @click="cancelEdit">取消</button>
        <button v-if="editMode === 'edit'" class="wv-btn wv-btn-primary" @click="saveScheme">
          保存
        </button>
        <button
          v-if="editMode === 'copy'"
          class="wv-btn wv-btn-primary"
          :disabled="!newSchemeName.trim()"
          @click="saveScheme"
        >
          保存为新主题
        </button>
      </template>
    </WeaselModal>
  </div>
</template>

<style scoped>
.theme-editor {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.tabs {
  display: flex;
  gap: 4px;
  background: var(--color-bg-active);
  padding: 4px;
  border-radius: 8px;
  width: fit-content;
}

.tab {
  padding: 6px 16px;
  border: none;
  background: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  color: var(--color-text-primary);
}

.tab.active {
  background: var(--color-bg-secondary);
  box-shadow: var(--shadow-sm);
}

.scheme-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 300px;
  overflow-y: auto;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.header-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.list-header h3 {
  margin: 0;
  font-size: 14px;
}

.btn-sm {
  padding: 4px 12px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
}

.scheme-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s;
}

.scheme-item:hover {
  background: var(--color-bg-hover);
}

.scheme-item.selected,
.scheme-item.is-active {
  background: var(--color-accent-light);
}

.scheme-item.is-pending {
  background: var(--color-pending-bg);
  border: 1px solid var(--color-pending);
}

.scheme-item.is-selected {
  outline: 2px solid var(--color-accent);
  outline-offset: -2px;
}

.scheme-color {
  width: 48px;
  height: 32px;
  border-radius: 4px;
  border: 1px solid var(--color-border);
}

.scheme-info {
  flex: 1;
}

.scheme-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.scheme-name {
  font-size: 14px;
  font-weight: 500;
}

.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
}

.badge-active {
  background: var(--color-success);
  color: var(--color-text-inverse);
}

.badge-pending {
  background: var(--color-pending);
  color: var(--color-text-inverse);
}

.action-btn {
  padding: 4px 8px;
  border: 1px solid var(--color-btn-default-border);
  background: var(--color-btn-default-bg);
  border-radius: 4px;
  font-size: 11px;
  cursor: pointer;
  white-space: nowrap;
  color: var(--color-text-primary);
}

.btn-apply {
  color: var(--color-accent);
  border-color: var(--color-accent);
}

.btn-apply:hover {
  background: var(--color-accent);
  color: var(--color-text-inverse);
}

.btn-cancel {
  color: var(--color-danger);
  border-color: var(--color-danger);
}

.btn-cancel:hover {
  background: var(--color-danger);
  color: var(--color-text-inverse);
}

.scheme-author {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.edit-btn {
  padding: 4px 8px;
  border: 1px solid var(--color-btn-default-border);
  background: var(--color-btn-default-bg);
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  color: var(--color-text-primary);
}

.preview {
  margin-top: 16px;
}

.candidate-window {
  margin-top: 8px;
  padding: 16px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
}

.candidate-bg {
  padding: 8px;
  border-radius: 8px;
  display: inline-flex;
  flex-direction: column;
  gap: 4px;
}

.candidate-item {
  display: flex;
  gap: 8px;
  padding: 4px 8px;
  border-radius: 4px;
}

.label {
  opacity: 0.6;
}

.wv-btn {
  padding: 8px 16px;
  border: 1px solid var(--color-btn-default-border);
  background: var(--color-btn-default-bg);
  border-radius: 6px;
  cursor: pointer;
  color: var(--color-text-primary);
}

.style-editor {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.color-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 16px;
}

.color-field label {
  display: block;
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-bottom: 4px;
}

.color-input {
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-input input[type='color'] {
  width: 32px;
  height: 32px;
  border: none;
  cursor: pointer;
}

.color-input span {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.style-section h4 {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: var(--color-text-primary);
  border-bottom: 1px solid var(--color-border);
  padding-bottom: 4px;
}

.style-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.style-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.style-field label {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.style-field input,
.style-field select {
  padding: 6px 8px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 13px;
  background: var(--color-bg-input);
  color: var(--color-text-primary);
}

.style-toggles {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}

.toggle-field {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  cursor: pointer;
}

.toggle-field input[type='checkbox'] {
  width: 16px;
  height: 16px;
}

.style-actions {
  display: flex;
  justify-content: flex-end;
  padding-top: 8px;
}

.pending-notice {
  background: var(--color-pending-bg);
  border-left: 4px solid var(--color-pending);
  padding: 12px 16px;
  margin-bottom: 12px;
  border-radius: 4px;
}

.notice-text {
  font-size: 13px;
  color: var(--color-warning-text);
}

.scheme-item.is-deleting {
  opacity: 0.5;
  background: var(--color-bg-tertiary);
  border: 1px dashed var(--color-border-dark);
}

.badge-deleting {
  background: var(--color-danger);
  color: var(--color-text-inverse);
}

.btn-delete {
  color: var(--color-danger);
  border-color: var(--color-danger);
}

.btn-delete:hover:not(:disabled) {
  background: var(--color-danger);
  color: var(--color-text-inverse);
}

.btn-delete:disabled {
  color: var(--color-text-tertiary);
  border-color: var(--color-border);
  cursor: not-allowed;
  opacity: 0.5;
}

.btn-cancel-delete {
  color: var(--color-success);
  border-color: var(--color-success);
}

.btn-cancel-delete:hover {
  background: var(--color-success);
  color: var(--color-text-inverse);
}
</style>
