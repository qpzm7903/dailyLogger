<template>
  <div
    v-if="!isOnline"
    class="flex items-center gap-2 px-3 py-1.5 bg-amber-900/80 border border-amber-700 rounded-lg text-amber-100 text-xs"
    role="status"
    :aria-label="t('networkStatus.offline')"
  >
    <span class="relative flex h-2 w-2">
      <span
        class="animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-400 opacity-75"
      ></span>
      <span class="relative inline-flex rounded-full h-2 w-2 bg-amber-500"></span>
    </span>
    <span>{{ t('networkStatus.offline') }}</span>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

defineProps<{
  isOnline: boolean
}>()
</script>
