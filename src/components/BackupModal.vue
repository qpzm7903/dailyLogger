<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[600px] max-h-[80vh] overflow-hidden border border-gray-700 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">数据备份与恢复</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-gray-700">
        <button
          @click="activeTab = 'backup'"
          :class="[
            'flex-1 px-4 py-3 text-sm font-medium transition-colors',
            activeTab === 'backup'
              ? 'text-primary border-b-2 border-primary'
              : 'text-gray-400 hover:text-white'
          ]"
        >
          创建备份
        </button>
        <button
          @click="activeTab = 'restore'"
          :class="[
            'flex-1 px-4 py-3 text-sm font-medium transition-colors',
            activeTab === 'restore'
              ? 'text-primary border-b-2 border-primary'
              : 'text-gray-400 hover:text-white'
          ]"
        >
          恢复数据
        </button>
        <button
          @click="activeTab = 'history'"
          :class="[
            'flex-1 px-4 py-3 text-sm font-medium transition-colors',
            activeTab === 'history'
              ? 'text-primary border-b-2 border-primary'
              : 'text-gray-400 hover:text-white'
          ]"
        >
          备份历史
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-6">
        <!-- Backup Tab -->
        <div v-if="activeTab === 'backup'" class="space-y-4">
          <div class="bg-darker rounded-lg p-4">
            <h3 class="text-sm font-medium text-gray-300 mb-2">备份说明</h3>
            <ul class="text-xs text-gray-400 space-y-1">
              <li>• 备份将包含数据库和所有截图文件</li>
              <li>• 备份文件保存为 ZIP 格式</li>
              <li>• 默认保存到 Documents/DailyLogger/backups/</li>
            </ul>
          </div>

          <div>
            <label class="text-xs text-gray-300 block mb-2">备份位置（可选）</label>
            <div class="flex gap-2">
              <input
                v-model="backupDir"
                type="text"
                placeholder="默认: Documents/DailyLogger/backups/"
                class="flex-1 bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
              <button
                @click="selectBackupDir"
                class="px-3 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg text-sm transition-colors"
              >
                选择
              </button>
            </div>
          </div>

          <button
            @click="createBackup"
            :disabled="isBackingUp"
            class="w-full py-3 bg-primary hover:bg-primary/80 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg font-medium transition-colors"
          >
            {{ isBackingUp ? '备份中...' : '创建备份' }}
          </button>

          <!-- Backup Result -->
          <div v-if="backupResult" class="bg-green-900/30 border border-green-700 rounded-lg p-4">
            <h4 class="text-sm font-medium text-green-400 mb-2">备份成功</h4>
            <div class="text-xs text-gray-300 space-y-1">
              <p>路径: {{ backupResult.path }}</p>
              <p>大小: {{ formatSize(backupResult.size_bytes) }}</p>
              <p>记录数: {{ backupResult.record_count }}</p>
              <p>截图数: {{ backupResult.screenshot_count }}</p>
            </div>
          </div>
        </div>

        <!-- Restore Tab -->
        <div v-if="activeTab === 'restore'" class="space-y-4">
          <div class="bg-darker rounded-lg p-4">
            <h3 class="text-sm font-medium text-gray-300 mb-2">恢复说明</h3>
            <ul class="text-xs text-gray-400 space-y-1">
              <li>• 选择要恢复的备份文件</li>
              <li>• 恢复前会自动备份当前数据</li>
              <li>• 恢复失败时可回滚到之前的状态</li>
            </ul>
          </div>

          <button
            @click="selectBackupFile"
            class="w-full py-3 bg-gray-700 hover:bg-gray-600 rounded-lg font-medium transition-colors"
          >
            选择备份文件
          </button>

          <!-- Selected Backup Info -->
          <div v-if="selectedBackup" class="bg-darker rounded-lg p-4">
            <h4 class="text-sm font-medium text-gray-300 mb-2">选择的备份</h4>
            <div class="text-xs text-gray-400 space-y-1">
              <p>创建时间: {{ formatDate(selectedBackup.created_at) }}</p>
              <p>大小: {{ formatSize(selectedBackup.size_bytes) }}</p>
              <p>记录数: {{ selectedBackup.record_count }}</p>
              <p>截图数: {{ selectedBackup.screenshot_count }}</p>
            </div>
          </div>

          <!-- Confirm Restore -->
          <div v-if="selectedBackup && !showConfirm" class="space-y-2">
            <button
              @click="showConfirm = true"
              class="w-full py-3 bg-red-600 hover:bg-red-700 rounded-lg font-medium transition-colors"
            >
              确认恢复
            </button>
          </div>

          <div v-if="showConfirm && selectedBackup" class="bg-red-900/30 border border-red-700 rounded-lg p-4">
            <h4 class="text-sm font-medium text-red-400 mb-2">⚠️ 确认恢复</h4>
            <p class="text-xs text-gray-300 mb-3">
              恢复操作将用备份数据替换当前数据。恢复前会自动备份当前数据，以便在恢复失败时回滚。
            </p>
            <div class="flex gap-2">
              <button
                @click="confirmRestore"
                :disabled="isRestoring"
                class="flex-1 py-2 bg-red-600 hover:bg-red-700 disabled:opacity-50 rounded-lg font-medium transition-colors"
              >
                {{ isRestoring ? '恢复中...' : '继续恢复' }}
              </button>
              <button
                @click="showConfirm = false"
                class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
              >
                取消
              </button>
            </div>
          </div>

          <!-- Restore Result -->
          <div v-if="restoreResult" class="bg-green-900/30 border border-green-700 rounded-lg p-4">
            <h4 class="text-sm font-medium text-green-400 mb-2">恢复成功</h4>
            <div class="text-xs text-gray-300 space-y-1">
              <p>记录数: {{ restoreResult.record_count }}</p>
              <p>截图数: {{ restoreResult.screenshot_count }}</p>
              <p v-if="restoreResult.auto_backup_created">已创建数据回滚备份</p>
            </div>
          </div>
        </div>

        <!-- History Tab -->
        <div v-if="activeTab === 'history'" class="space-y-4">
          <div v-if="isLoadingBackups" class="text-center py-8 text-gray-400">
            加载中...
          </div>

          <div v-else-if="backups.length === 0" class="text-center py-8 text-gray-400">
            暂无备份记录
          </div>

          <div v-else class="space-y-2">
            <div
              v-for="backup in backups"
              :key="backup.path"
              class="bg-darker rounded-lg p-4 flex items-center justify-between"
            >
              <div class="flex-1">
                <div class="text-sm font-medium text-gray-200">
                  {{ formatDate(backup.created_at) }}
                </div>
                <div class="text-xs text-gray-400 mt-1">
                  {{ formatSize(backup.size_bytes) }} • {{ backup.record_count }} 条记录 • {{ backup.screenshot_count }} 张截图
                </div>
              </div>
              <div class="flex gap-2">
                <button
                  @click="restoreFromHistory(backup)"
                  class="px-3 py-1.5 text-xs bg-primary hover:bg-primary/80 rounded transition-colors"
                >
                  恢复
                </button>
                <button
                  @click="deleteBackupFile(backup.path)"
                  class="px-3 py-1.5 text-xs bg-red-700 hover:bg-red-600 rounded transition-colors"
                >
                  删除
                </button>
              </div>
            </div>
          </div>

          <button
            @click="loadBackups"
            class="w-full py-2 bg-gray-700 hover:bg-gray-600 rounded-lg text-sm transition-colors"
          >
            刷新
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const emit = defineEmits(['close'])

const activeTab = ref('backup')
const backupDir = ref('')
const isBackingUp = ref(false)
const backupResult = ref(null)

const selectedBackup = ref(null)
const showConfirm = ref(false)
const isRestoring = ref(false)
const restoreResult = ref(null)

const isLoadingBackups = ref(false)
const backups = ref([])

// Load backups on mount
onMounted(() => {
  loadBackups()
})

async function loadBackups() {
  isLoadingBackups.value = true
  try {
    backups.value = await invoke('list_backups')
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
      title: '选择备份目录'
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
    backupResult.value = await invoke('create_backup', { backupDir: dir })
    await loadBackups()
  } catch (e) {
    console.error('Failed to create backup:', e)
    alert('备份失败: ' + e)
  } finally {
    isBackingUp.value = false
  }
}

async function selectBackupFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Backup', extensions: ['zip'] }],
      title: '选择备份文件'
    })
    if (selected) {
      selectedBackup.value = await invoke('get_backup_info', { backupPath: selected })
    }
  } catch (e) {
    console.error('Failed to select backup file:', e)
  }
}

async function confirmRestore() {
  if (!selectedBackup.value) return

  isRestoring.value = true
  try {
    restoreResult.value = await invoke('restore_backup', { backupPath: selectedBackup.value.path })
    showConfirm.value = false
    selectedBackup.value = null
  } catch (e) {
    console.error('Failed to restore backup:', e)
    alert('恢复失败: ' + e)
  } finally {
    isRestoring.value = false
  }
}

async function restoreFromHistory(backup) {
  selectedBackup.value = backup
  activeTab.value = 'restore'
}

async function deleteBackupFile(path) {
  if (!confirm('确定要删除这个备份吗？')) return

  try {
    await invoke('delete_backup', { backupPath: path })
    await loadBackups()
  } catch (e) {
    console.error('Failed to delete backup:', e)
    alert('删除失败: ' + e)
  }
}

function formatSize(bytes) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}

function formatDate(isoString) {
  try {
    const date = new Date(isoString)
    return date.toLocaleString('zh-CN')
  } catch {
    return isoString
  }
}
</script>
