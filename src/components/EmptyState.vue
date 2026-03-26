<template>
  <div class="flex flex-col items-center justify-center py-8 text-gray-400">
    <!-- SVG Illustration -->
    <div class="mb-4 opacity-60">
      <!-- Screenshots Empty -->
      <svg v-if="type === 'screenshots'" width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="10" y="15" width="60" height="45" rx="4" stroke="currentColor" stroke-width="2" fill="none"/>
        <circle cx="28" cy="32" r="5" stroke="currentColor" stroke-width="2"/>
        <path d="M10 50 L30 38 L45 48 L60 35 L70 42" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M55 10 L60 15 M60 10 L55 15" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>

      <!-- Daily Report Empty -->
      <svg v-else-if="type === 'dailyReport'" width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="15" y="10" width="50" height="60" rx="4" stroke="currentColor" stroke-width="2" fill="none"/>
        <path d="M25 25 H55" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        <path d="M25 35 H55" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        <path d="M25 45 H45" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        <rect x="25" y="52" width="30" height="10" rx="2" stroke="currentColor" stroke-width="2" fill="none"/>
      </svg>

      <!-- Search Results Empty -->
      <svg v-else-if="type === 'searchResults'" width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="35" cy="35" r="20" stroke="currentColor" stroke-width="2" fill="none"/>
        <path d="M50 50 L65 65" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>
        <path d="M28 32 L32 36 M32 32 L28 36" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        <path d="M25 55 Q35 48 45 55" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/>
      </svg>
    </div>

    <!-- Text Description -->
    <p class="text-sm text-center max-w-xs">
      <slot>{{ defaultMessage }}</slot>
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  type: 'screenshots' | 'dailyReport' | 'searchResults'
}>()

const defaultMessage = computed(() => {
  switch (props.type) {
    case 'screenshots':
      return '暂无截图记录'
    case 'dailyReport':
      return '尚未生成日报'
    case 'searchResults':
      return '未找到匹配结果'
    default:
      return '暂无数据'
  }
})
</script>
