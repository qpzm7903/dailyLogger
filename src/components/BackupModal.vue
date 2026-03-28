<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-[var(--color-surface-1)] rounded-2xl w-[600px] max-h-[80vh] overflow-hidden border border-[var(--color-border)] flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('backup.title') }}</h2>
        <button @click="$emit('close')" class="text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">✕</button>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-[var(--color-border)]">
        <button
          @click="activeTab = 'backup'"
          :class="[
            'flex-1 px-4 py-3 text-sm font-medium transition-colors',
            activeTab === 'backup'
              ? 'text-primary border-b-2 border-primary'
              : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'
          ]"
        >
          {{ t('backup.tabCreateBackup') }}
        </button>
        <button
          @click="activeTab = 'restore'"
          :class="[
            'flex-1 px-4 py-3 text-sm font-medium transition-colors',
            activeTab === 'restore'
              ? 'text-primary border-b-2 border-primary'
              : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'
          ]"
        >
          {{ t('backup.tabRestore') }}
        </button>
        <button
          @click="activeTab = 'history'"
          :class="[
            'flex-1 px-4 py-3 text-sm font-medium transition-colors',
            activeTab === 'history'
              ? 'text-primary border-b-2 border-primary'
              : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'
          ]"
        >
          {{ t('backup.tabHistory') }}
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-6">
        <!-- Backup Tab -->
        <div v-if="activeTab === 'backup'" class="space-y-4">
          <div class="bg-[var(--color-surface-0)] rounded-lg p-4">
            <h3 class="text-sm font-medium text-gray-300 mb-2">{{ t('backup.backupInfo') }}</h3>
            <ul class="text-xs text-[var(--color-text-secondary)] space-y-1">
              <li>{{ t('backup.backupIncludes') }}</li>
              <li>{{ t('backup.backupFormat') }}</li>
              <li>{{ t('backup.backupLocation') }}</li>
            </ul>
          </div>

          <div>
            <label class="text-xs text-gray-300 block mb-2">{{ t('backup.backupPath') }}</label>
            <div class="flex gap-2">
              <input
                v-model="backupDir"
                type="text"
                :placeholder="t('backup.backupPathPlaceholder')"
                class="flex-1 bg-[var(--color-surface-0)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-[var(--color-text-muted)] focus:border-primary focus:outline-none"
              />
              <button
                @click="selectBackupDir"
                class="px-3 py-2 bg-[var(--color-action-secondary)] hover:bg-[var(--color-action-neutral)] rounded-lg text-sm transition-colors"
              >
                {{ t('backup.select') }}
              </button>
            </div>
          </div>

          <button
            @click="createBackup"
            :disabled="isBackingUp"
            class="w-full py-3 bg-primary hover:bg-primary/80 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg font-medium transition-colors"
          >
            {{ isBackingUp ? t('backup.backingUp') : t('backup.createBackup') }}
          </button>

          <!-- Backup Result -->
          <div v-if="backupResult" class="bg-green-900/30 border border-green-700 rounded-lg p-4">
            <h4 class="text-sm font-medium text-green-400 mb-2">{{ t('backup.backupSuccess') }}</h4>
            <div class="text-xs text-gray-300 space-y-1">
              <p>{{ t('backup.path') }} {{ backupResult.path }}</p>
              <p>{{ t('backup.size') }} {{ formatSize(backupResult.size_bytes) }}</p>
              <p>{{ t('backup.recordCount') }} {{ backupResult.record_count }}</p>
              <p>{{ t('backup.screenshotCount') }} {{ backupResult.screenshot_count }}</p>
            </div>
          </div>
        </div>

        <!-- Restore Tab -->
        <div v-if="activeTab === 'restore'" class="space-y-4">
          <div class="bg-[var(--color-surface-0)] rounded-lg p-4">
            <h3 class="text-sm font-medium text-gray-300 mb-2">{{ t('backup.restoreInfo') }}</h3>
            <ul class="text-xs text-[var(--color-text-secondary)] space-y-1">
              <li>{{ t('backup.restoreIncludes') }}</li>
              <li>{{ t('backup.restoreAutoBackup') }}</li>
              <li>{{ t('backup.restoreRollback') }}</li>
            </ul>
          </div>

          <button
            @click="selectBackupFile"
            class="w-full py-3 bg-[var(--color-action-secondary)] hover:bg-[var(--color-action-neutral)] rounded-lg font-medium transition-colors"
          >
            {{ t('backup.selectBackupFile') }}
          </button>

          <!-- Selected Backup Info -->
          <div v-if="selectedBackup" class="bg-[var(--color-surface-0)] rounded-lg p-4">
            <h4 class="text-sm font-medium text-gray-300 mb-2">{{ t('backup.selectedBackup') }}</h4>
            <div class="text-xs text-[var(--color-text-secondary)] space-y-1">
              <p>{{ t('backup.createdAt') }} {{ formatDate(selectedBackup.created_at) }}</p>
              <p>{{ t('backup.size') }} {{ formatSize(selectedBackup.size_bytes) }}</p>
              <p>{{ t('backup.recordCount') }} {{ selectedBackup.record_count }}</p>
              <p>{{ t('backup.screenshotCount') }} {{ selectedBackup.screenshot_count }}</p>
            </div>
          </div>

          <!-- Confirm Restore -->
          <div v-if="selectedBackup && !showConfirm" class="space-y-2">
            <button
              @click="showConfirm = true"
              class="w-full py-3 bg-red-600 hover:bg-red-700 rounded-lg font-medium transition-colors"
            >
              {{ t('backup.confirmRestore') }}
            </button>
          </div>

          <div v-if="showConfirm && selectedBackup" class="bg-red-900/30 border border-red-700 rounded-lg p-4">
            <h4 class="text-sm font-medium text-red-400 mb-2">{{ t('backup.confirmRestoreTitle') }}</h4>
            <p class="text-xs text-gray-300 mb-3">
              {{ t('backup.confirmRestoreMessage') }}
            </p>
            <div class="flex gap-2">
              <button
                @click="confirmRestore"
                :disabled="isRestoring"
                class="flex-1 py-2 bg-red-600 hover:bg-red-700 disabled:opacity-50 rounded-lg font-medium transition-colors"
              >
                {{ isRestoring ? t('backup.restoring') : t('backup.continueRestore') }}
              </button>
              <button
                @click="showConfirm = false"
                class="px-4 py-2 bg-[var(--color-action-secondary)] hover:bg-[var(--color-action-neutral)] rounded-lg transition-colors"
              >
                {{ t('common.cancel') }}
              </button>
            </div>
          </div>

          <!-- Restore Result -->
          <div v-if="restoreResult" class="bg-green-900/30 border border-green-700 rounded-lg p-4">
            <h4 class="text-sm font-medium text-green-400 mb-2">{{ t('backup.restoreSuccess') }}</h4>
            <div class="text-xs text-gray-300 space-y-1">
              <p>{{ t('backup.recordCount') }} {{ restoreResult.record_count }}</p>
              <p>{{ t('backup.screenshotCount') }} {{ restoreResult.screenshot_count }}</p>
              <p v-if="restoreResult.auto_backup_created">{{ t('backup.autoBackupCreated') }}</p>
            </div>
          </div>
        </div>

        <!-- History Tab -->
        <div v-if="activeTab === 'history'" class="space-y-4">
          <div v-if="isLoadingBackups" class="text-center py-8 text-[var(--color-text-secondary)]">
            {{ t('backup.loading') }}
          </div>

          <div v-else-if="backups.length === 0" class="text-center py-8 text-[var(--color-text-secondary)]">
            {{ t('backup.noBackups') }}
          </div>

          <div v-else class="space-y-2">
            <div
              v-for="backup in backups"
              :key="backup.path"
              class="bg-[var(--color-surface-0)] rounded-lg p-4 flex items-center justify-between"
            >
              <div class="flex-1">
                <div class="text-sm font-medium text-gray-200">
                  {{ formatDate(backup.created_at) }}
                </div>
                <div class="text-xs text-[var(--color-text-secondary)] mt-1">
                  {{ formatSize(backup.size_bytes) }} • {{ backup.record_count }} {{ t('backup.recordsWithCount', { count: backup.record_count }).split(' ')[1] }} • {{ backup.screenshot_count }} {{ t('backup.screenshotsWithCount', { count: backup.screenshot_count }).split(' ')[1] }}
                </div>
              </div>
              <div class="flex gap-2">
                <button
                  @click="restoreFromHistory(backup)"
                  class="px-3 py-1.5 text-xs bg-primary hover:bg-primary/80 rounded transition-colors"
                >
                  {{ t('backup.tabRestore') }}
                </button>
                <button
                  @click="deleteBackupFile(backup.path)"
                  class="px-3 py-1.5 text-xs bg-red-700 hover:bg-red-600 rounded transition-colors"
                >
                  {{ t('common.delete') }}
                </button>
              </div>
            </div>
          </div>

          <button
            @click="loadBackups"
            class="w-full py-2 bg-[var(--color-action-secondary)] hover:bg-[var(--color-action-neutral)] rounded-lg text-sm transition-colors"
          >
            {{ t('backup.refresh') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { systemActions } from '../features/system/actions'

interface BackupInfo {
  path: string
  created_at: string
  size_bytes: number
  record_count: number
  screenshot_count: number
}

interface BackupResult {
  path: string
  size_bytes: number
  record_count: number
  screenshot_count: number
}

interface RestoreResult {
  record_count: number
  screenshot_count: number
  auto_backup_created: boolean
}

const { t } = useI18n()
const emit = defineEmits<{(e: 'close'): void}>()

const activeTab = ref<'backup' | 'restore' | 'history'>('backup')
const backupDir = ref('')
const isBackingUp = ref(false)
const backupResult = ref<BackupResult | null>(null)

const selectedBackup = ref<BackupInfo | null>(null)
const showConfirm = ref(false)
const isRestoring = ref(false)
const restoreResult = ref<RestoreResult | null>(null)

const isLoadingBackups = ref(false)
const backups = ref<BackupInfo[]>([])

// Load backups on mount
onMounted(() => {
  loadBackups()
})

async function loadBackups() {
  isLoadingBackups.value = true
  try {
    backups.value = await invoke<BackupInfo[]>('list_backups')
  } catch (e) {
    console.error('Failed to load backups:', e)
  } finally {
    isLoadingBackups.value = false
  }
}

async function selectBackupDir() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('backup.select')
    })
    if (selected) {
      backupDir.value = selected
    }
  } catch (e) {
    console.error('Failed to select directory:', e)
  }
}

async function createBackup() {
  isBackingUp.value = true
  backupResult.value = null
  try {
    const dir = backupDir.value || null
    backupResult.value = await invoke<BackupResult>('create_backup', { backupDir: dir })
    await loadBackups()
  } catch (e) {
    console.error('Failed to create backup:', e)
    alert(t('backup.backupFailed', { error: e }))
  } finally {
    isBackingUp.value = false
  }
}

async function selectBackupFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Backup', extensions: ['zip'] }],
      title: t('backup.selectBackupFile')
    })
    if (selected) {
      selectedBackup.value = await invoke<BackupInfo>('get_backup_info', { backupPath: selected })
    }
  } catch (e) {
    console.error('Failed to select backup file:', e)
  }
}

async function confirmRestore() {
  if (!selectedBackup.value) return

  isRestoring.value = true
  try {
    restoreResult.value = await invoke<RestoreResult>('restore_backup', { backupPath: selectedBackup.value.path })
    showConfirm.value = false
    selectedBackup.value = null
  } catch (e) {
    console.error('Failed to restore backup:', e)
    alert(t('backup.restoreFailed', { error: e }))
  } finally {
    isRestoring.value = false
  }
}

async function restoreFromHistory(backup: BackupInfo) {
  selectedBackup.value = backup
  activeTab.value = 'restore'
}

async function deleteBackupFile(path: string) {
  if (!confirm(t('backup.confirmDeleteBackup'))) return

  try {
    await systemActions.deleteBackup(path)
    await loadBackups()
  } catch (e) {
    console.error('Failed to delete backup:', e)
    alert(t('backup.deleteFailed', { error: e }))
  }
}

function formatSize(bytes: number) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}

function formatDate(isoString: string) {
  try {
    const date = new Date(isoString)
    return date.toLocaleString('zh-CN')
  } catch {
    return isoString
  }
}
</script>
