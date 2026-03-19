<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] h-[90vh] max-w-4xl overflow-hidden border border-gray-700 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('historyViewer.title') }}</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Filters -->
      <div class="px-6 py-3 border-b border-gray-700 flex items-center gap-4 flex-wrap">
        <div class="flex items-center gap-2">
          <label class="text-sm text-gray-400">{{ t('historyViewer.startDate') }}</label>
          <input
            type="date"
            v-model="startDate"
            class="bg-darker border border-gray-600 rounded px-2 py-1 text-sm text-white focus:border-primary focus:outline-none"
          />
        </div>
        <div class="flex items-center gap-2">
          <label class="text-sm text-gray-400">{{ t('historyViewer.endDate') }}</label>
          <input
            type="date"
            v-model="endDate"
            class="bg-darker border border-gray-600 rounded px-2 py-1 text-sm text-white focus:border-primary focus:outline-none"
          />
        </div>
        <div class="flex items-center gap-2">
          <label class="text-sm text-gray-400">{{ t('historyViewer.source') }}</label>
          <select
            v-model="sourceType"
            class="bg-darker border border-gray-600 rounded px-2 py-1 text-sm text-white focus:border-primary focus:outline-none"
          >
            <option :value="null">{{ t('historyViewer.all') }}</option>
            <option value="auto">{{ t('historyViewer.autoCapture') }}</option>
            <option value="manual">{{ t('historyViewer.manualRecord') }}</option>
          </select>
        </div>
        <button
          @click="loadRecords"
          :disabled="isLoading"
          class="px-4 py-1 bg-primary text-white rounded text-sm hover:bg-primary/80 transition-colors disabled:opacity-50"
        >
          {{ isLoading ? t('historyViewer.loading') : t('historyViewer.query') }}
        </button>
        <span v-if="records.length > 0" class="text-sm text-gray-400 ml-auto">
          {{ t('historyViewer.totalRecords', { count: records.length }) }}
        </span>
      </div>

      <!-- Tag Filter -->
      <div class="px-6 py-3 border-b border-gray-700">
        <TagFilter
          ref="tagFilterRef"
          v-model="selectedTags"
        />
      </div>

      <!-- Record List -->
      <div class="flex-1 overflow-auto p-4" ref="scrollContainer" @scroll="handleScroll">
        <div v-if="isLoading && records.length === 0" class="text-center py-8 text-gray-500">
          {{ t('historyViewer.loading') }}
        </div>
        <div v-else-if="records.length === 0" class="text-center py-8 text-gray-500">
          {{ t('historyViewer.noRecords') }}
        </div>

        <div v-else class="flex flex-col divide-y divide-gray-700">
          <div
            v-for="record in records"
            :key="record.id"
            class="py-3 px-2 hover:bg-darker/50 transition-colors group"
          >
            <div class="flex items-start justify-between gap-2">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <span
                    :class="record.source_type === 'auto' ? 'bg-blue-500/20 text-blue-400' : 'bg-green-500/20 text-green-400'"
                    class="px-2 py-0.5 rounded text-xs"
                  >
                    {{ record.source_type === 'auto' ? t('historyViewer.auto') : t('historyViewer.manual') }}
                  </span>
                  <span class="text-xs text-gray-500">{{ formatTime(record.timestamp) }}</span>
                </div>
                <p class="text-sm text-gray-300 truncate">{{ truncateContent(record.content) }}</p>
                <!-- Manual tags -->
                <div v-if="getRecordTags(record.id).length > 0" class="flex flex-wrap gap-1 mt-2">
                  <TagBadge
                    v-for="tag in getRecordTags(record.id)"
                    :key="tag.id"
                    :tag="tag"
                  />
                </div>
              </div>
              <button
                @click="confirmDelete(record)"
                class="opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-300 text-sm px-2 py-1 transition-opacity"
              >
                {{ t('historyViewer.delete') }}
              </button>
              <button
                v-if="currentUser"
                @click="openShareModal(record)"
                class="opacity-0 group-hover:opacity-100 text-primary hover:text-primary/80 text-sm px-2 py-1 transition-opacity"
              >
                {{ t('team.share') }}
              </button>
            </div>
          </div>
        </div>

        <!-- Loading indicator for pagination -->
        <div v-if="isLoadingMore" class="text-center py-4 text-gray-500">
          {{ t('historyViewer.loadingMore') }}
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div
      v-if="recordToDelete"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-60"
    >
      <div class="bg-dark rounded-xl p-6 max-w-sm border border-gray-700">
        <h3 class="text-lg font-semibold mb-4">{{ t('historyViewer.confirmDelete') }}</h3>
        <p class="text-gray-400 mb-6">{{ t('historyViewer.confirmDeleteMessage') }}</p>
        <div class="flex justify-end gap-3">
          <button
            @click="recordToDelete = null"
            class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors"
          >
            {{ t('historyViewer.cancel') }}
          </button>
          <button
            @click="deleteRecord"
            :disabled="isDeleting"
            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-400 transition-colors disabled:opacity-50"
          >
            {{ isDeleting ? t('historyViewer.deleting') : t('historyViewer.delete') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Share Record Modal -->
    <div
      v-if="showShareModal && recordToShare"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-60"
    >
      <div class="bg-dark rounded-xl p-6 max-w-sm w-full mx-4 border border-gray-700">
        <h3 class="text-lg font-semibold mb-4">{{ t('team.shareRecord') }}</h3>
        <p class="text-gray-400 text-sm mb-4 truncate">
          {{ truncateContent(recordToShare.content) }}
        </p>

        <div v-if="isLoadingTeams" class="text-center py-4 text-gray-500">
          {{ t('team.loadingTeams') }}
        </div>
        <div v-else-if="userTeams.length === 0" class="text-center py-4 text-gray-500">
          {{ t('team.noTeamsToShare') }}
        </div>
        <div v-else class="space-y-2 max-h-60 overflow-auto">
          <button
            v-for="teamWithMembers in userTeams"
            :key="teamWithMembers.team.id"
            @click="shareRecord(teamWithMembers.team.id)"
            :disabled="isSharing"
            class="w-full text-left px-4 py-3 bg-darker rounded-lg hover:bg-gray-700 transition-colors disabled:opacity-50"
          >
            <div class="font-medium">{{ teamWithMembers.team.name }}</div>
            <div class="text-xs text-gray-500 mt-1">
              {{ t('team.members') }}: {{ teamWithMembers.members.length }}
            </div>
          </button>
        </div>

        <div class="flex justify-end gap-3 mt-6">
          <button
            @click="closeShareModal"
            class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-500 transition-colors"
          >
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showSuccess, showError } from '../stores/toast'
import TagFilter from './TagFilter.vue'
import TagBadge from './TagBadge.vue'

const { t } = useI18n()

const emit = defineEmits(['close'])

const props = defineProps({
  initialTag: {
    type: Object,
    default: null
  },
  currentUser: {
    type: Object,
    default: null
  }
})

// State
const startDate = ref('')
const endDate = ref('')
const sourceType = ref(null)
const selectedTags = ref([])
const records = ref([])
const recordTags = ref({}) // Map of record id to tags
const isLoading = ref(false)
const isLoadingMore = ref(false)
const page = ref(0)
const pageSize = 50
const hasMore = ref(true)
const scrollContainer = ref(null)
const recordToDelete = ref(null)
const isDeleting = ref(false)
const tagFilterRef = ref(null)

// Share state
const recordToShare = ref(null)
const isSharing = ref(false)
const showShareModal = ref(false)
const userTeams = ref([])
const isLoadingTeams = ref(false)

// Initialize dates to last 7 days
onMounted(() => {
  const end = new Date()
  const start = new Date()
  start.setDate(start.getDate() - 7)

  endDate.value = formatDate(end)
  startDate.value = formatDate(start)

  // Apply initial tag filter from TagCloud selection (FIX-003)
  if (props.initialTag) {
    selectedTags.value = [props.initialTag]
  }

  loadRecords()
})

// Watch for tag filter changes
watch(selectedTags, () => {
  loadRecords()
}, { deep: true })

function formatDate(date) {
  return date.toISOString().split('T')[0]
}

function formatTime(timestamp) {
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function truncateContent(content) {
  if (!content) return ''
  // Try to parse JSON content
  try {
    const parsed = JSON.parse(content)
    if (parsed.summary) return parsed.summary
    if (parsed.note) return parsed.note
    return content
  } catch {
    return content.length > 100 ? content.slice(0, 100) + '...' : content
  }
}

async function loadRecords() {
  isLoading.value = true
  page.value = 0
  records.value = []
  recordTags.value = {}
  hasMore.value = true

  try {
    // If tags are selected, use tag-based filtering
    if (selectedTags.value.length > 0) {
      const tagIds = selectedTags.value.map(t => t.id)
      const result = await invoke('get_records_by_manual_tags', {
        tagIds: tagIds,
        startDate: startDate.value,
        endDate: endDate.value,
        sourceType: sourceType.value
      })

      records.value = result
      hasMore.value = false // Tag-based query doesn't support pagination
    } else {
      // Regular date/source filtering
      const result = await invoke('get_history_records', {
        startDate: startDate.value,
        endDate: endDate.value,
        sourceType: sourceType.value,
        page: 0,
        pageSize: pageSize
      })

      records.value = result
      hasMore.value = result.length === pageSize
    }

    // Load tags for all records
    await loadRecordTags()
  } catch (error) {
    showError(t('historyViewer.loadFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

// Load tags for all displayed records (batch query - PERF-001)
async function loadRecordTags() {
  const ids = records.value.map(r => r.id)
  if (ids.length === 0) return

  try {
    const tagsMap = await invoke('get_tags_for_records', { recordIds: ids })
    recordTags.value = tagsMap
  } catch (e) {
    console.error('Failed to load tags for records:', e)
    recordTags.value = {}
  }
}

// Get tags for a specific record
function getRecordTags(recordId) {
  return recordTags.value[recordId] || []
}

async function loadMoreRecords() {
  if (isLoadingMore.value || !hasMore.value) return
  if (selectedTags.value.length > 0) return // Tag-based query doesn't support pagination

  isLoadingMore.value = true
  page.value += 1

  try {
    const result = await invoke('get_history_records', {
      startDate: startDate.value,
      endDate: endDate.value,
      sourceType: sourceType.value,
      page: page.value,
      pageSize: pageSize
    })

    records.value.push(...result)
    hasMore.value = result.length === pageSize

    // Load tags for new records (batch)
    const newIds = result.map(r => r.id)
    if (newIds.length > 0) {
      try {
        const tagsMap = await invoke('get_tags_for_records', { recordIds: newIds })
        Object.assign(recordTags.value, tagsMap)
      } catch (e) {
        console.error('Failed to load tags for new records:', e)
      }
    }
  } catch (error) {
    showError(t('historyViewer.loadMoreFailed', { error }))
    page.value -= 1 // Revert page increment on error
  } finally {
    isLoadingMore.value = false
  }
}

function handleScroll() {
  if (!scrollContainer.value) return

  const { scrollTop, scrollHeight, clientHeight } = scrollContainer.value
  const isNearBottom = scrollHeight - scrollTop - clientHeight < 100

  if (isNearBottom && hasMore.value && !isLoadingMore.value) {
    loadMoreRecords()
  }
}

function confirmDelete(record) {
  recordToDelete.value = record
}

async function deleteRecord() {
  if (!recordToDelete.value) return

  isDeleting.value = true

  try {
    await invoke('delete_record', { id: recordToDelete.value.id })

    // Remove from local list
    records.value = records.value.filter(r => r.id !== recordToDelete.value.id)

    showSuccess(t('historyViewer.recordDeleted'))
    recordToDelete.value = null
  } catch (error) {
    showError(t('historyViewer.deleteFailed', { error }))
  } finally {
    isDeleting.value = false
  }
}

// Share record functions
async function openShareModal(record) {
  if (!props.currentUser) return

  recordToShare.value = record
  showShareModal.value = true
  isLoadingTeams.value = true

  try {
    const teams = await invoke('get_user_teams', { userId: props.currentUser.id })
    userTeams.value = teams
  } catch (error) {
    showError(t('team.shareFailed', { error }))
    closeShareModal()
  } finally {
    isLoadingTeams.value = false
  }
}

function closeShareModal() {
  showShareModal.value = false
  recordToShare.value = null
  userTeams.value = []
}

async function shareRecord(teamId) {
  if (!recordToShare.value || !props.currentUser) return

  isSharing.value = true

  try {
    await invoke('share_record_to_team', {
      params: {
        teamId: teamId,
        recordId: recordToShare.value.id
      },
      currentUserId: props.currentUser.id
    })

    showSuccess(t('team.recordShared'))
    closeShareModal()
  } catch (error) {
    const errorMsg = error.toString()
    if (errorMsg.includes('already shared')) {
      showError(t('team.alreadyShared'))
    } else {
      showError(t('team.shareFailed', { error }))
    }
  } finally {
    isSharing.value = false
  }
}
</script>