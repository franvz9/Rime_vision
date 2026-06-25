<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['changed'])

interface RimeColor {
  r: number
  g: number
  b: number
  a: number
}

interface ColorScheme {
  name: string
  author: string
  color_space: string
  back_color: RimeColor | null
  border_color: RimeColor | null
  text_color: RimeColor | null
  hilited_text_color: RimeColor | null
  hilited_back_color: RimeColor | null
  hilited_candidate_back_color: RimeColor | null
  candidate_text_color: RimeColor | null
  hilited_candidate_text_color: RimeColor | null
  candidate_label_color: RimeColor | null
  hilited_candidate_label_color: RimeColor | null
  comment_text_color: RimeColor | null
  hilited_comment_text_color: RimeColor | null
  preedit_back_color: RimeColor | null
  candidate_back_color: RimeColor | null
}

interface StyleData {
  style: any
  light_schemes: Record<string, ColorScheme>
  dark_schemes: Record<string, ColorScheme>
  selected_light: string
  selected_dark: string
}

const styleData = ref<StyleData | null>(null)
const activeTab = ref<'light' | 'dark' | 'style'>('light')
const selectedScheme = ref<string>('')
const editingScheme = ref<ColorScheme | null>(null)
const showEditor = ref(false)

const schemes = computed(() => {
  if (!styleData.value) return {}
  return activeTab.value === 'light'
    ? styleData.value.light_schemes
    : styleData.value.dark_schemes
})

onMounted(async () => {
  try {
    styleData.value = await invoke('get_style_data')
    selectedScheme.value = activeTab.value === 'light'
      ? styleData.value!.selected_light
      : styleData.value!.selected_dark
  } catch (e) {
    console.error('Failed to load style data:', e)
  }
})

function selectScheme(name: string) {
  selectedScheme.value = name
  emit('changed')
}

function editScheme(name: string) {
  const scheme = schemes.value[name]
  if (scheme) {
    editingScheme.value = { ...scheme }
    showEditor.value = true
  }
}

async function saveScheme() {
  if (!editingScheme.value) return
  try {
    await invoke('save_color_scheme', {
      name: editingScheme.value.name,
      scheme: editingScheme.value,
    })
    showEditor.value = false
    styleData.value = await invoke('get_style_data')
  } catch (e) {
    console.error('Failed to save scheme:', e)
  }
}

function parseColor(v: any): RimeColor | null {
  if (!v) return null
  if (typeof v === 'object' && 'r' in v) return v as RimeColor
  if (typeof v === 'string') return hexToRgb(v)
  return null
}

function colorToRgba(v: any): string {
  const c = parseColor(v)
  if (!c) return 'transparent'
  return `rgba(${c.r}, ${c.g}, ${c.b}, ${c.a / 255})`
}

function getColorHex(v: any): string {
  const c = parseColor(v)
  if (!c) return '#000000'
  return rgbToHex(c)
}

function getHexDisplay(v: any): string {
  const c = parseColor(v)
  if (!c) return ''
  if (c.a < 255) {
    return `0x${c.a.toString(16).padStart(2, '0')}${c.b.toString(16).padStart(2, '0')}${c.g.toString(16).padStart(2, '0')}${c.r.toString(16).padStart(2, '0')}`
  }
  return `0x${c.b.toString(16).padStart(2, '0')}${c.g.toString(16).padStart(2, '0')}${c.r.toString(16).padStart(2, '0')}`
}

function setColorHex(scheme: any, key: string, hex: string) {
  const rgb = hexToRgb(hexToRimeHex(hex))
  if (rgb) {
    scheme[key] = rgb
    emit('changed')
  }
}
</script>

<template>
  <div class="theme-editor">
    <div class="tabs">
      <button :class="['tab', { active: activeTab === 'light' }]" @click="activeTab = 'light'">亮色主题</button>
      <button :class="['tab', { active: activeTab === 'dark' }]" @click="activeTab = 'dark'">暗色主题</button>
      <button :class="['tab', { active: activeTab === 'style' }]" @click="activeTab = 'style'">样式</button>
    </div>

    <div class="scheme-list">
      <div
        v-for="(scheme, name) in schemes"
        :key="name"
        :class="['scheme-item', { selected: selectedScheme === name }]"
        @click="selectScheme(name as string)"
      >
        <div class="scheme-color" :style="{ background: colorToRgba(scheme.back_color) }"></div>
        <div class="scheme-info">
          <div class="scheme-name">{{ name }}</div>
          <div v-if="scheme.author" class="scheme-author">{{ scheme.author }}</div>
        </div>
        <button class="edit-btn" @click.stop="editScheme(name as string)">编辑</button>
      </div>
    </div>

    <!-- Simple preview -->
    <div class="preview">
      <h3>预览</h3>
      <div class="candidate-window" v-if="selectedScheme && schemes[selectedScheme]">
        <div
          class="candidate-bg"
          :style="{ background: colorToRgba(schemes[selectedScheme].back_color) }"
        >
          <div
            class="candidate-item"
            :style="{
              background: colorToRgba(schemes[selectedScheme].hilited_candidate_back_color),
              color: colorToRgba(schemes[selectedScheme].hilited_candidate_text_color)
            }"
          >
            <span class="label">1.</span>
            <span class="text">候选词</span>
          </div>
          <div class="candidate-item" :style="{ color: colorToRgba(schemes[selectedScheme].candidate_text_color) }">
            <span class="label" :style="{ color: colorToRgba(schemes[selectedScheme].candidate_label_color) }">2.</span>
            <span class="text">示例</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Simple editor modal -->
    <div v-if="showEditor" class="modal-overlay" @click.self="showEditor = false">
      <div class="modal">
        <h3>编辑配色: {{ editingScheme?.name }}</h3>
        <div class="form-group">
          <label>作者</label>
          <input v-model="editingScheme!.author" />
        </div>
        <div class="color-grid">
          <div class="color-field" v-for="(label, key) in {
            'back_color': '背景色',
            'text_color': '文字色',
            'candidate_text_color': '候选文字',
            'hilited_candidate_back_color': '高亮背景',
            'hilited_candidate_text_color': '高亮文字',
            'label_color': '编号色',
            'comment_text_color': '注释色'
          }" :key="key">
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
        <div class="modal-actions">
          <button class="btn" @click="showEditor = false">取消</button>
          <button class="btn btn-primary" @click="saveScheme">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
function hexToRgb(hex: string): { r: number; g: number; b: number; a: number } | null {
  if (!hex) return null
  const cleaned = hex.replace(' ', '')
  const match8 = cleaned.match(/^0x([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$/)
  if (match8) {
    return {
      a: parseInt(match8[1], 16),
      b: parseInt(match8[2], 16),
      g: parseInt(match8[3], 16),
      r: parseInt(match8[4], 16),
    }
  }
  const match6 = cleaned.match(/^0x([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$/)
  if (match6) {
    return {
      b: parseInt(match6[1], 16),
      g: parseInt(match6[2], 16),
      r: parseInt(match6[3], 16),
      a: 255,
    }
  }
  return null
}

function rgbToHex(c: { r: number; g: number; b: number } | null): string {
  if (!c) return '#000000'
  return `#${c.r.toString(16).padStart(2, '0')}${c.g.toString(16).padStart(2, '0')}${c.b.toString(16).padStart(2, '0')}`
}

function hexToRimeHex(hex: string, a: number = 255): string {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  if (a < 255) {
    return `0x${a.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${r.toString(16).padStart(2, '0')}`
  }
  return `0x${b.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${r.toString(16).padStart(2, '0')}`
}

</script>

<style scoped>
.theme-editor {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.tabs {
  display: flex;
  gap: 4px;
  background: #e9e9eb;
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
}

.tab.active {
  background: white;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.scheme-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 300px;
  overflow-y: auto;
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
  background: #f0f0f0;
}

.scheme-item.selected {
  background: #e3f2fd;
}

.scheme-color {
  width: 48px;
  height: 32px;
  border-radius: 4px;
  border: 1px solid #ddd;
}

.scheme-info {
  flex: 1;
}

.scheme-name {
  font-size: 14px;
  font-weight: 500;
}

.scheme-author {
  font-size: 12px;
  color: #666;
}

.edit-btn {
  padding: 4px 8px;
  border: 1px solid #ddd;
  background: white;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.preview {
  margin-top: 16px;
}

.candidate-window {
  margin-top: 8px;
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

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: white;
  border-radius: 12px;
  padding: 24px;
  width: 500px;
  max-height: 80vh;
  overflow-y: auto;
}

.modal h3 {
  margin-bottom: 16px;
}

.form-group {
  margin-bottom: 12px;
}

.form-group label {
  display: block;
  font-size: 13px;
  color: #666;
  margin-bottom: 4px;
}

.form-group input {
  width: 100%;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
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
  color: #666;
  margin-bottom: 4px;
}

.color-input {
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-input input[type="color"] {
  width: 32px;
  height: 32px;
  border: none;
  cursor: pointer;
}

.color-input span {
  font-size: 12px;
  color: #999;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn {
  padding: 8px 16px;
  border: 1px solid #ddd;
  background: white;
  border-radius: 6px;
  cursor: pointer;
}

.btn-primary {
  background: #007aff;
  color: white;
  border: none;
}
</style>
