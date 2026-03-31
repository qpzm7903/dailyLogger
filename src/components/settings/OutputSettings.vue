<template>
  <div class="space-y-6">
    <!-- Obsidian Vaults -->
    <div>
      <h3 class="text-sm font-medium text-[var(--color-text-secondary)] mb-3">{{ $t('settings.outputConfig') }}</h3>
      <div class="space-y-3">
        <label class="text-xs text-[var(--color-text-secondary)] block">{{ $t('settings.obsidianVaults') }}</label>
        <!-- Vault list -->
        <div v-for="(vault, index) in localVaults" :key="vault.path"
          class="bg-[var(--color-surface-0)] border border-[var(--color-border)] rounded-lg px-3 py-2">
          <div class="flex items-center gap-2">
            <button @click="setDefaultVault(index)" class="text-xs shrink-0"
              :class="vault.is_default ? 'text-primary font-bold' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]'">
              {{ vault.is_default ? '★' : '☆' }}
            </button>
            <div class="flex-1 min-w-0">
              <div class="text-sm text-[var(--color-text-primary)] truncate">{{ vault.name }}</div>
              <div class="text-xs text-[var(--color-text-muted)] truncate">{{ vault.path }}</div>
            </div>
            <button @click="removeVault(index)" class="text-[var(--color-text-muted)] hover:text-red-400 text-xs shrink-0">✕</button>
          </div>
          <!-- Window patterns input for auto-detection -->
          <div class="mt-2 ml-6">
            <input
              :value="getVaultPatternsString(vault)"
              @input="updateVaultPatterns(vault, ($event.target as HTMLInputElement).value)"
              type="text"
              :placeholder="$t('settings.windowPatternsPlaceholder') || '窗口标题匹配模式，如: VS Code, project-A'"
              class="w-full bg-[var(--color-surface-1)] border border-[var(--color-border-subtle)] rounded px-2 py-1 text-xs text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] focus:border-primary focus:outline-none"
            />
            <div class="text-xs text-[var(--color-text-muted)] mt-1">{{ $t('settings.windowPatternsHint') || '多个模式用逗号分隔' }}</div>
          </div>
        </div>
        <div v-if="vaults.length === 0" class="text-xs text-[var(--color-text-muted)] py-2">
          {{ $t('settings.noVaultConfigured') }}
        </div>
        <!-- Add vault form -->
        <div class="flex gap-2">
          <input v-model="newVaultName" type="text" :placeholder="$t('common.name')"
            class="w-1/3 bg-[var(--color-surface-0)] border border-[var(--color-border)] rounded-lg px-2 py-1.5 text-xs text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] focus:border-primary focus:outline-none" />
          <input v-model="newVaultPath" type="text" :placeholder="$t('common.path')"
            class="flex-1 bg-[var(--color-surface-0)] border border-[var(--color-border)] rounded-lg px-2 py-1.5 text-xs text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] focus:border-primary focus:outline-none" />
          <button @click="addVault" :disabled="!newVaultName.trim() || !newVaultPath.trim()"
            class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-30 rounded-lg text-xs text-primary transition-colors shrink-0">
            {{ $t('common.add') }}
          </button>
        </div>
        <!-- Auto-detect vault by window toggle -->
        <div class="flex items-center gap-2 mt-3 pt-3 border-t border-[var(--color-border)]">
          <input
            v-model="localSettings.auto_detect_vault_by_window"
            type="checkbox"
            id="auto-detect-vault"
            class="w-4 h-4 rounded border-[var(--color-border-subtle)] bg-[var(--color-surface-1)] text-primary focus:ring-primary focus:ring-offset-0"
          />
          <label for="auto-detect-vault" class="text-xs text-[var(--color-text-secondary)]">
            {{ $t('settings.autoDetectVaultByWindow') || '根据窗口标题自动选择 Vault' }}
          </label>
        </div>
      </div>
    </div>

    <!-- Logseq Graphs -->
    <div>
      <label class="text-xs text-[var(--color-text-secondary)] block mb-2">{{ $t('settings.logseqGraphs') }}</label>
      <!-- Graph list -->
      <div v-for="(graph, index) in graphs" :key="graph.path"
        class="flex items-center gap-2 bg-[var(--color-surface-0)] border border-[var(--color-border)] rounded-lg px-3 py-2 mb-2">
        <button @click="setDefaultGraph(index)" class="text-xs shrink-0"
          :class="graph.is_default ? 'text-primary font-bold' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]'">
          {{ graph.is_default ? '★' : '☆' }}
        </button>
        <div class="flex-1 min-w-0">
          <div class="text-sm text-[var(--color-text-primary)] truncate">{{ graph.name }}</div>
          <div class="text-xs text-[var(--color-text-muted)] truncate">{{ graph.path }}</div>
        </div>
        <button @click="removeGraph(index)" class="text-[var(--color-text-muted)] hover:text-red-400 text-xs shrink-0">✕</button>
      </div>
      <div v-if="graphs.length === 0" class="text-xs text-[var(--color-text-muted)] py-2 mb-2">
        {{ $t('settings.noGraphConfigured') }}
      </div>
      <!-- Add graph form -->
      <div class="flex gap-2">
        <input v-model="newGraphName" type="text" :placeholder="$t('common.name')"
          class="w-1/3 bg-[var(--color-surface-0)] border border-[var(--color-border)] rounded-lg px-2 py-1.5 text-xs text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] focus:border-primary focus:outline-none" />
        <input v-model="newGraphPath" type="text" :placeholder="$t('common.path')"
          class="flex-1 bg-[var(--color-surface-0)] border border-[var(--color-border)] rounded-lg px-2 py-1.5 text-xs text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] focus:border-primary focus:outline-none" />
        <button @click="addGraph" :disabled="!newGraphName.trim() || !newGraphPath.trim()"
          class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-30 rounded-lg text-xs text-primary transition-colors shrink-0">
          {{ $t('common.add') }}
        </button>
      </div>
    </div>



    <!-- Debug Tools -->
    <div>
      <h3 class="text-sm font-medium text-[var(--color-text-secondary)] mb-3">{{ $t('settings.debugTools') }}</h3>
      <div class="space-y-3">
        <button
          @click="exportLogs"
          :disabled="isExportingLogs"
          class="w-full px-4 py-2 bg-[var(--color-action-neutral)] hover:bg-[var(--color-action-neutral)] disabled:opacity-50 rounded-lg text-sm text-[var(--color-text-secondary)] transition-colors flex items-center justify-center gap-2"
        >
          {{ isExportingLogs ? $t('settings.exporting') : '📤 ' + $t('settings.exportLogs') }}
        </button>
        <span v-if="exportError" class="text-xs text-red-400 block">{{ exportError }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showError, showSuccess } from '../../stores/toast'
import { systemActions } from '../../features/system/actions'

// Props
interface Vault {
  name: string
  path: string
  is_default?: boolean
  window_patterns?: string[]
}

interface Graph {
  name: string
  path: string
  is_default?: boolean
}

interface Props {
  settings: {
    auto_detect_vault_by_window: boolean
  }
  vaults: Vault[]
  graphs: Graph[]
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  (e: 'update:settings', value: Props['settings']): void
  (e: 'update:vaults', value: Vault[]): void
  (e: 'update:graphs', value: Graph[]): void
}>()

// Composables
const { t } = useI18n()

// Local state
const localSettings = ref({ ...props.settings })
const localVaults = ref<Vault[]>([...props.vaults])
const localGraphs = ref<Graph[]>([...props.graphs])

// Input state
const newVaultName = ref('')
const newVaultPath = ref('')
const newGraphName = ref('')
const newGraphPath = ref('')



// Export state
const isExportingLogs = ref(false)
const exportError = ref('')

// Flag to prevent bidirectional watch loop
let isUpdatingFromProps = false

// Watch for external changes
watch(() => props.settings, (newVal) => {
  isUpdatingFromProps = true
  localSettings.value = { ...newVal }
  nextTick(() => { isUpdatingFromProps = false })
}, { deep: true })

watch(() => props.vaults, (newVal) => {
  isUpdatingFromProps = true
  localVaults.value = [...newVal]
  nextTick(() => { isUpdatingFromProps = false })
}, { deep: true })

watch(() => props.graphs, (newVal) => {
  localGraphs.value = [...newVal]
}, { deep: true })

// Watch for local changes and emit
watch(localSettings, (newVal) => {
  if (!isUpdatingFromProps) {
    emit('update:settings', newVal)
  }
}, { deep: true })

// Vault management methods
function addVault() {
  if (newVaultName.value.trim() && newVaultPath.value.trim()) {
    localVaults.value.push({
      name: newVaultName.value.trim(),
      path: newVaultPath.value.trim(),
      is_default: localVaults.value.length === 0
    })
    emit('update:vaults', [...localVaults.value])
    newVaultName.value = ''
    newVaultPath.value = ''
  }
}

function removeVault(index: number) {
  localVaults.value.splice(index, 1)
  emit('update:vaults', [...localVaults.value])
}

function setDefaultVault(index: number) {
  localVaults.value.forEach((v, i) => {
    v.is_default = i === index
  })
  emit('update:vaults', [...localVaults.value])
}

// Vault patterns management (for auto-detection by window title)
function getVaultPatternsString(vault: Vault): string {
  if (!vault.window_patterns || vault.window_patterns.length === 0) return ''
  return vault.window_patterns.join(', ')
}

function updateVaultPatterns(vault: Vault, patternsStr: string) {
  const patterns = patternsStr
    .split(',')
    .map(s => s.trim())
    .filter(s => s.length > 0)
  vault.window_patterns = patterns.length > 0 ? patterns : undefined
  emit('update:vaults', [...localVaults.value])
}

// Watch for local vault changes and emit
watch(localVaults, (newVal) => {
  if (!isUpdatingFromProps) {
    emit('update:vaults', [...newVal])
  }
}, { deep: true })

// Graph management methods
function addGraph() {
  if (newGraphName.value.trim() && newGraphPath.value.trim()) {
    localGraphs.value.push({
      name: newGraphName.value.trim(),
      path: newGraphPath.value.trim(),
      is_default: localGraphs.value.length === 0
    })
    emit('update:graphs', [...localGraphs.value])
    newGraphName.value = ''
    newGraphPath.value = ''
  }
}

function removeGraph(index: number) {
  localGraphs.value.splice(index, 1)
  emit('update:graphs', [...localGraphs.value])
}

function setDefaultGraph(index: number) {
  localGraphs.value.forEach((g, i) => {
    g.is_default = i === index
  })
  emit('update:graphs', [...localGraphs.value])
}


// Export logs
async function exportLogs() {
  isExportingLogs.value = true
  exportError.value = ''

  try {
    await systemActions.exportLogs()
    showSuccess(t('settings.logsExported'))
  } catch (err) {
    exportError.value = String(err)
    showError(err)
  } finally {
    isExportingLogs.value = false
  }
}
</script>