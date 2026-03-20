<template>
  <div class="space-y-6">
    <!-- Obsidian Vaults -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.outputConfig') }}</h3>
      <div class="space-y-3">
        <label class="text-xs text-gray-300 block">{{ $t('settings.obsidianVaults') }}</label>
        <!-- Vault list -->
        <div v-for="(vault, index) in vaults" :key="index"
          class="flex items-center gap-2 bg-darker border border-gray-700 rounded-lg px-3 py-2">
          <button @click="setDefaultVault(index)" class="text-xs shrink-0"
            :class="vault.is_default ? 'text-primary font-bold' : 'text-gray-500 hover:text-gray-300'">
            {{ vault.is_default ? '★' : '☆' }}
          </button>
          <div class="flex-1 min-w-0">
            <div class="text-sm text-gray-100 truncate">{{ vault.name }}</div>
            <div class="text-xs text-gray-500 truncate">{{ vault.path }}</div>
          </div>
          <button @click="removeVault(index)" class="text-gray-500 hover:text-red-400 text-xs shrink-0">✕</button>
        </div>
        <div v-if="vaults.length === 0" class="text-xs text-gray-500 py-2">
          {{ $t('settings.noVaultConfigured') }}
        </div>
        <!-- Add vault form -->
        <div class="flex gap-2">
          <input v-model="newVaultName" type="text" :placeholder="$t('common.name')"
            class="w-1/3 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
          <input v-model="newVaultPath" type="text" :placeholder="$t('common.path')"
            class="flex-1 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
          <button @click="addVault" :disabled="!newVaultName.trim() || !newVaultPath.trim()"
            class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-30 rounded-lg text-xs text-primary transition-colors shrink-0">
            {{ $t('common.add') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Logseq Graphs -->
    <div>
      <label class="text-xs text-gray-300 block mb-2">{{ $t('settings.logseqGraphs') }}</label>
      <!-- Graph list -->
      <div v-for="(graph, index) in graphs" :key="index"
        class="flex items-center gap-2 bg-darker border border-gray-700 rounded-lg px-3 py-2 mb-2">
        <button @click="setDefaultGraph(index)" class="text-xs shrink-0"
          :class="graph.is_default ? 'text-primary font-bold' : 'text-gray-500 hover:text-gray-300'">
          {{ graph.is_default ? '★' : '☆' }}
        </button>
        <div class="flex-1 min-w-0">
          <div class="text-sm text-gray-100 truncate">{{ graph.name }}</div>
          <div class="text-xs text-gray-500 truncate">{{ graph.path }}</div>
        </div>
        <button @click="removeGraph(index)" class="text-gray-500 hover:text-red-400 text-xs shrink-0">✕</button>
      </div>
      <div v-if="graphs.length === 0" class="text-xs text-gray-500 py-2 mb-2">
        {{ $t('settings.noGraphConfigured') }}
      </div>
      <!-- Add graph form -->
      <div class="flex gap-2">
        <input v-model="newGraphName" type="text" :placeholder="$t('common.name')"
          class="w-1/3 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
        <input v-model="newGraphPath" type="text" :placeholder="$t('common.path')"
          class="flex-1 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
        <button @click="addGraph" :disabled="!newGraphName.trim() || !newGraphPath.trim()"
          class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-30 rounded-lg text-xs text-primary transition-colors shrink-0">
          {{ $t('common.add') }}
        </button>
      </div>
    </div>

    <!-- Notion Integration -->
    <div>
      <label class="text-xs text-gray-300 block mb-2">{{ $t('settings.notionIntegration') }}</label>
      <div class="space-y-3">
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.notionApiKey') }}</label>
          <input v-model="localSettings.notion_api_key" type="password" :placeholder="$t('settings.notionApiKeyPlaceholder')"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
        </div>
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.notionDatabaseId') }}</label>
          <input v-model="localSettings.notion_database_id" type="text" :placeholder="$t('settings.notionDatabaseIdPlaceholder')"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
        </div>
        <div class="flex gap-2">
          <button @click="testNotionConnection" :disabled="isTestingNotionConnection"
            class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-50 rounded-lg text-xs text-primary transition-colors">
            {{ isTestingNotionConnection ? $t('common.testing') : $t('common.testConnection') }}
          </button>
          <span v-if="notionConnectionStatus" class="text-xs"
            :class="notionConnectionStatus === 'success' ? 'text-green-400' : 'text-red-400'">
            {{ notionConnectionStatus === 'success' ? '✓ ' + $t('common.connected') : '✗ ' + $t('common.failed') }}
          </span>
        </div>
        <p class="text-xs text-gray-500">
          {{ $t('settings.notionHint') }}
        </p>
      </div>
    </div>

    <!-- GitHub Work Time Statistics -->
    <div>
      <label class="text-xs text-gray-300 block mb-2">{{ $t('settings.githubWorkTime') }}</label>
      <div class="space-y-3">
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.githubToken') }}</label>
          <input v-model="localSettings.github_token" type="password" :placeholder="$t('settings.githubTokenPlaceholder')"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
        </div>
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.githubRepos') }}</label>
          <textarea
            v-model="githubReposText"
            rows="3"
            placeholder="owner/repo1&#10;owner/repo2"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-none font-mono"
          />
        </div>
        <div class="flex gap-2">
          <button @click="testGithubConnection" :disabled="isTestingGithubConnection"
            class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-50 rounded-lg text-xs text-primary transition-colors">
            {{ isTestingGithubConnection ? $t('common.testing') : $t('common.testConnection') }}
          </button>
          <span v-if="githubConnectionStatus" class="text-xs"
            :class="githubConnectionStatus === 'success' ? 'text-green-400' : 'text-red-400'">
            {{ githubConnectionStatus === 'success' ? '✓ ' + $t('common.connected') : '✗ ' + $t('common.failed') }}
          </span>
        </div>
        <p class="text-xs text-gray-500">
          {{ $t('settings.githubHint') }}
        </p>
      </div>
    </div>

    <!-- Slack Notification -->
    <div>
      <label class="text-xs text-gray-300 block mb-2">{{ $t('settings.slackNotification') }}</label>
      <div class="space-y-3">
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.slackWebhookUrl') }}</label>
          <input v-model="localSettings.slack_webhook_url" type="password" :placeholder="$t('settings.slackWebhookPlaceholder')"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
        </div>
        <div class="flex gap-2">
          <button @click="testSlackConnection" :disabled="isTestingSlackConnection"
            class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-50 rounded-lg text-xs text-primary transition-colors">
            {{ isTestingSlackConnection ? $t('common.testing') : $t('common.testConnection') }}
          </button>
          <span v-if="slackConnectionStatus" class="text-xs"
            :class="slackConnectionStatus === 'success' ? 'text-green-400' : 'text-red-400'">
            {{ slackConnectionStatus === 'success' ? '✓ ' + $t('common.connected') : '✗ ' + $t('common.failed') }}
          </span>
        </div>
        <p class="text-xs text-gray-500">
          {{ $t('settings.slackHint') }}
        </p>
      </div>
    </div>

    <!-- Debug Tools -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.debugTools') }}</h3>
      <div class="space-y-3">
        <button
          @click="exportLogs"
          :disabled="isExportingLogs"
          class="w-full px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:opacity-50 rounded-lg text-sm text-gray-200 transition-colors flex items-center justify-center gap-2"
        >
          {{ isExportingLogs ? $t('settings.exporting') : '📤 ' + $t('settings.exportLogs') }}
        </button>
        <span v-if="exportError" class="text-xs text-red-400 block">{{ exportError }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showError, showSuccess } from '@/stores/toast'

// Props
interface Vault {
  name: string
  path: string
  is_default?: boolean
}

interface Graph {
  name: string
  path: string
  is_default?: boolean
}

interface Props {
  settings: {
    notion_api_key: string | null
    notion_database_id: string | null
    github_token: string | null
    github_repositories: string
    slack_webhook_url: string | null
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
const githubReposText = ref('')

// Connection test state
const isTestingNotionConnection = ref(false)
const notionConnectionStatus = ref<'success' | 'failed' | null>(null)
const isTestingGithubConnection = ref(false)
const githubConnectionStatus = ref<'success' | 'failed' | null>(null)
const isTestingSlackConnection = ref(false)
const slackConnectionStatus = ref<'success' | 'failed' | null>(null)

// Export state
const isExportingLogs = ref(false)
const exportError = ref('')

// Watch for external changes
watch(() => props.settings, (newVal) => {
  localSettings.value = { ...newVal }
}, { deep: true })

watch(() => props.vaults, (newVal) => {
  localVaults.value = [...newVal]
}, { deep: true })

watch(() => props.graphs, (newVal) => {
  localGraphs.value = [...newVal]
}, { deep: true })

// Watch for local changes and emit
watch(localSettings, (newVal) => {
  emit('update:settings', newVal)
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

// Connection test methods
async function testNotionConnection() {
  if (!localSettings.value.notion_api_key || !localSettings.value.notion_database_id) {
    showError(t('settings.notionConfigRequired'))
    return
  }

  isTestingNotionConnection.value = true
  notionConnectionStatus.value = null

  try {
    const result = await invoke<{ success: boolean; message: string }>('test_notion_connection', {
      apiKey: localSettings.value.notion_api_key,
      databaseId: localSettings.value.notion_database_id
    })

    notionConnectionStatus.value = result.success ? 'success' : 'failed'
    if (result.success) {
      showSuccess(t('settings.notionConnectionSuccess'))
    } else {
      showError(result.message)
    }
  } catch (err) {
    notionConnectionStatus.value = 'failed'
    showError(err)
  } finally {
    isTestingNotionConnection.value = false
  }
}

async function testGithubConnection() {
  if (!localSettings.value.github_token) {
    showError(t('settings.githubTokenRequired'))
    return
  }

  isTestingGithubConnection.value = true
  githubConnectionStatus.value = null

  try {
    const result = await invoke<{ success: boolean; message: string }>('test_github_connection', {
      token: localSettings.value.github_token
    })

    githubConnectionStatus.value = result.success ? 'success' : 'failed'
    if (result.success) {
      showSuccess(t('settings.githubConnectionSuccess'))
    } else {
      showError(result.message)
    }
  } catch (err) {
    githubConnectionStatus.value = 'failed'
    showError(err)
  } finally {
    isTestingGithubConnection.value = false
  }
}

async function testSlackConnection() {
  if (!localSettings.value.slack_webhook_url) {
    showError(t('settings.slackWebhookRequired'))
    return
  }

  isTestingSlackConnection.value = true
  slackConnectionStatus.value = null

  try {
    const result = await invoke<{ success: boolean; message: string }>('test_slack_webhook', {
      webhookUrl: localSettings.value.slack_webhook_url
    })

    slackConnectionStatus.value = result.success ? 'success' : 'failed'
    if (result.success) {
      showSuccess(t('settings.slackConnectionSuccess'))
    } else {
      showError(result.message)
    }
  } catch (err) {
    slackConnectionStatus.value = 'failed'
    showError(err)
  } finally {
    isTestingSlackConnection.value = false
  }
}

// Export logs
async function exportLogs() {
  isExportingLogs.value = true
  exportError.value = ''

  try {
    await invoke('export_logs')
    showSuccess(t('settings.logsExported'))
  } catch (err) {
    exportError.value = String(err)
    showError(err)
  } finally {
    isExportingLogs.value = false
  }
}
</script>