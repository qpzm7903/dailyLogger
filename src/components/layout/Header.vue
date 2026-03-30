<template>
  <header
    :class="!isOnline ? 'mt-9' : ''"
    class="bg-[var(--color-surface-1)]/80 backdrop-blur-md border-b border-[var(--color-border)]/50 px-6 py-3 flex items-center justify-between transition-[margin] duration-300"
  >
    <!-- Left: Title & Status -->
    <div class="flex items-center gap-4">
      <h1 class="text-lg font-semibold text-[var(--color-text-primary)]">DailyLogger</h1>

      <!-- Auto Capture Status Indicator -->
      <div class="flex items-center gap-2 px-3 py-1 rounded-full bg-surface-1/50 border border-[var(--color-border)]/30">
        <span
          :class="autoCaptureEnabled ? 'bg-status-success animate-pulse' : 'bg-gray-500'"
          class="w-2 h-2 rounded-full inline-block transition-colors duration-300"
        ></span>
        <span class="text-xs text-[var(--color-text-secondary)]">
          {{ autoCaptureEnabled ? t('header.running') : t('header.paused') }}
        </span>
      </div>

      <!-- Today's Record Count -->
      <div class="flex items-center gap-1.5 px-3 py-1 rounded-full bg-surface-1/50 border border-[var(--color-border)]/30">
        <span class="text-sm">📝</span>
        <span class="text-xs text-[var(--color-text-secondary)]">
          {{ todayRecordsCount }} {{ t('header.records') }}
        </span>
      </div>
    </div>

    <!-- Right: Status & Time -->
    <div class="flex items-center gap-4">
      <!-- Pending Sync Badge -->
      <button
        v-if="offlineQueueCount > 0"
        @click="$emit('showOfflineQueue')"
        class="flex items-center gap-1.5 px-2.5 py-1 bg-yellow-500/20 text-yellow-400 rounded-full text-xs cursor-pointer hover:bg-yellow-500/30 transition-colors"
      >
        <span class="w-2 h-2 rounded-full bg-yellow-400 inline-block animate-pulse"></span>
        {{ t('header.pendingSync', { count: offlineQueueCount }) }}
      </button>

      <!-- Current Time -->
      <span class="text-sm text-[var(--color-text-secondary)] font-mono">{{ currentTime }}</span>
    </div>
  </header>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  isOnline: boolean
  offlineQueueCount: number
  currentTime: string
  autoCaptureEnabled: boolean
  todayRecordsCount: number
}>()

const emit = defineEmits<{
  showOfflineQueue: []
}>()

const { t } = useI18n()
</script>