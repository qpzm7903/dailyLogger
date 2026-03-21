<template>
  <header
    :class="!isOnline ? 'mt-9' : ''"
    class="bg-dark/80 backdrop-blur-md border-b border-gray-700/50 px-6 py-3 flex items-center justify-between transition-[margin] duration-300"
  >
    <!-- Left: Title -->
    <div class="flex items-center gap-3">
      <h1 class="text-lg font-semibold text-white">DailyLogger</h1>
    </div>

    <!-- Right: Status & Time -->
    <div class="flex items-center gap-4">
      <!-- Pending Sync Badge -->
      <div
        v-if="offlineQueueCount > 0"
        class="flex items-center gap-1.5 px-2.5 py-1 bg-yellow-500/20 text-yellow-400 rounded-full text-xs"
      >
        <span class="w-2 h-2 rounded-full bg-yellow-400 inline-block animate-pulse"></span>
        {{ t('header.pendingSync', { count: offlineQueueCount }) }}
      </div>

      <!-- Current Time -->
      <span class="text-sm text-gray-400 font-mono">{{ currentTime }}</span>
    </div>
  </header>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  isOnline: boolean
  offlineQueueCount: number
  currentTime: string
}>()

const { t } = useI18n()
</script>