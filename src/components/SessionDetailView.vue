<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-[var(--color-surface-1)] rounded-2xl max-w-2xl w-full max-h-[90vh] overflow-hidden border border-[var(--color-border)]">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('sessionDetailView.title') }}</h2>
        <button @click="$emit('close')" class="text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">✕</button>
      </div>

      <!-- Content -->
      <div class="p-6 overflow-auto max-h-[70vh]">
        <!-- Session Time -->
        <div class="mb-4 p-4 bg-[var(--color-surface-0)] rounded-lg">
          <div class="flex items-center gap-2 mb-2">
            <span class="text-lg">⏱️</span>
            <span class="text-sm text-[var(--color-text-secondary)]">{{ t('sessionDetailView.sessionTime') }}</span>
          </div>
          <p class="text-[var(--color-text-primary)] font-medium">
            {{ formatTime(session.start_time) }}
            <span class="text-[var(--color-text-muted)] mx-2">—</span>
            {{ session.end_time ? formatTime(session.end_time) : t('sessionDetailView.ongoing') }}
          </p>
        </div>

        <!-- Summary Section -->
        <div class="p-4 bg-[var(--color-surface-0)] rounded-lg border border-[var(--color-border)]">
          <div class="flex items-center justify-between mb-2">
            <label class="text-sm text-[var(--color-text-secondary)]">
              {{ hasUserSummary ? t('sessionDetailView.userSummary') : t('sessionDetailView.aiSummary') }}
            </label>
            <span v-if="hasUserSummary" class="text-xs text-green-400">
              ✓ {{ t('sessionDetailView.userEdited') }}
            </span>
          </div>

          <!-- Display Mode -->
          <div v-if="!isEditing">
            <p class="text-sm text-[var(--color-text-secondary)] whitespace-pre-wrap mb-3">
              {{ displaySummary }}
            </p>
            <button
              @click="startEditing"
              class="px-3 py-1.5 text-xs rounded-md bg-blue-600 hover:bg-blue-700 text-white transition-colors"
            >
              {{ t('sessionDetailView.editSummary') }}
            </button>
          </div>

          <!-- Edit Mode -->
          <div v-else>
            <textarea
              v-model="editingSummary"
              :placeholder="t('sessionDetailView.userSummaryPlaceholder')"
              class="w-full bg-[var(--color-surface-1)] border border-[var(--color-border-subtle)] rounded-lg p-3 text-sm text-[var(--color-text-secondary)] resize-none focus:outline-none focus:border-blue-500"
              rows="4"
            ></textarea>
            <div class="flex justify-end gap-2 mt-3">
              <button
                @click="cancelEditing"
                class="px-3 py-1.5 text-xs rounded-md bg-[var(--color-surface-2)] hover:bg-[var(--color-action-neutral)] text-[var(--color-text-primary)] transition-colors"
              >
                {{ t('sessionDetailView.cancel') }}
              </button>
              <button
                @click="saveSummary"
                :disabled="isSaving"
                class="px-3 py-1.5 text-xs rounded-md transition-colors"
                :class="isSaving
                  ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                  : 'bg-green-600 hover:bg-green-700 text-white'"
              >
                {{ isSaving ? t('sessionDetailView.saving') : t('sessionDetailView.save') }}
              </button>
            </div>
          </div>
        </div>

        <!-- AI Summary (if user has edited) -->
        <div v-if="hasUserSummary && session.ai_summary" class="mt-4 p-4 bg-[var(--color-surface-0)] rounded-lg border border-[var(--color-border)]">
          <label class="text-sm text-[var(--color-text-secondary)] mb-2 block">{{ t('sessionDetailView.aiSummary') }}</label>
          <p class="text-sm text-[var(--color-text-muted)] whitespace-pre-wrap">{{ session.ai_summary }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { showToast } from '../stores/toast'
import { sessionActions, type Session } from '../features/sessions/actions'

const { t, locale } = useI18n()

const props = defineProps<{
  session: Session
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'updated', session: Session): void
}>()

const isEditing = ref(false)
const editingSummary = ref('')
const originalUserSummary = ref(props.session.user_summary || '')
const isSaving = ref(false)

const hasUserSummary = computed(() => !!props.session.user_summary && props.session.user_summary.length > 0)

const displaySummary = computed(() => {
  if (hasUserSummary.value) {
    return props.session.user_summary
  }
  if (props.session.ai_summary) {
    return props.session.ai_summary
  }
  return t('sessionDetailView.noSummary')
})

const formatTime = (timestamp: string) => {
  const date = new Date(timestamp)
  return date.toLocaleString(locale.value === 'zh-CN' ? 'zh-CN' : 'en-US', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false
  })
}

const startEditing = () => {
  editingSummary.value = props.session.user_summary || props.session.ai_summary || ''
  originalUserSummary.value = props.session.user_summary || ''
  isEditing.value = true
}

const cancelEditing = () => {
  isEditing.value = false
  editingSummary.value = originalUserSummary.value
}

const saveSummary = async () => {
  if (isSaving.value) return

  isSaving.value = true
  try {
    await sessionActions.updateSessionUserSummary(
      props.session.id,
      editingSummary.value || null || ''
    )

    const updatedSession = {
      ...props.session,
      user_summary: editingSummary.value || null
    }

    showToast(t('sessionDetailView.summarySaved'), { type: 'success' })
    emit('updated', updatedSession)
    isEditing.value = false
  } catch (err) {
    const errorMsg = String(err)
    showToast(t('sessionDetailView.summarySaveFailed', { error: errorMsg }), { type: 'error' })
  } finally {
    isSaving.value = false
  }
}
</script>
