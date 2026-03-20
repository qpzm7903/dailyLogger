<template>
  <Transition name="banner">
    <div
      v-if="showBanner"
      :class="justReconnected ? 'bg-green-600' : 'bg-yellow-600'"
      class="fixed top-0 left-0 right-0 z-50 py-2 px-4 flex items-center justify-center gap-2 text-white text-sm"
    >
      <span v-if="justReconnected">✓ {{ t('offlineBanner.reconnected') }}</span>
      <span v-else>⚠ {{ t('offlineBanner.offline') }}</span>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps<{
  isOnline: boolean
}>()

const showBanner = ref(!props.isOnline)
const justReconnected = ref(false)
let reconnectTimer: ReturnType<typeof setTimeout> | null = null

watch(
  () => props.isOnline,
  (online) => {
    if (reconnectTimer) clearTimeout(reconnectTimer)

    if (!online) {
      justReconnected.value = false
      showBanner.value = true
    } else if (showBanner.value) {
      // Was offline, now reconnected — show green banner for 3s
      justReconnected.value = true
      reconnectTimer = setTimeout(() => {
        showBanner.value = false
        justReconnected.value = false
      }, 3000)
    }
  }
)
</script>

<style scoped>
.banner-enter-active,
.banner-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}

.banner-enter-from,
.banner-leave-to {
  transform: translateY(-100%);
  opacity: 0;
}
</style>
