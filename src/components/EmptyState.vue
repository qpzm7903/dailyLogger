<template>
  <div class="flex flex-col items-center justify-center py-8 text-gray-400">
    <!-- SVG illustrations for different empty states -->
    <div class="mb-4 opacity-60">
      <!-- Screenshots empty state -->
      <svg v-if="type === 'screenshots'" width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="10" y="15" width="60" height="45" rx="4" stroke="currentColor" stroke-width="2" fill="none"/>
        <circle cx="28" cy="32" r="6" stroke="currentColor" stroke-width="2" fill="none"/>
        <path d="M10 52 L28 38 L38 48 L52 34 L70 50" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
        <rect x="55" y="5" width="15" height="10" rx="2" stroke="currentColor" stroke-width="2" fill="none"/>
      </svg>

      <!-- Daily report empty state -->
      <svg v-else-if="type === 'dailyReport'" width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="12" y="8" width="56" height="64" rx="4" stroke="currentColor" stroke-width="2" fill="none"/>
        <rect x="20" y="16" width="20" height="4" rx="1" fill="currentColor" opacity="0.3"/>
        <rect x="20" y="24" width="40" height="2" rx="1" fill="currentColor" opacity="0.2"/>
        <rect x="20" y="30" width="36" height="2" rx="1" fill="currentColor" opacity="0.2"/>
        <rect x="20" y="36" width="32" height="2" rx="1" fill="currentColor" opacity="0.2"/>
        <rect x="20" y="46" width="16" height="16" rx="2" stroke="currentColor" stroke-width="2" fill="none"/>
        <path d="M24 54 L28 58 L34 50" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>

      <!-- Search results empty state -->
      <svg v-else-if="type === 'searchResults'" width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="35" cy="35" r="20" stroke="currentColor" stroke-width="2" fill="none"/>
        <line x1="50" y1="50" x2="65" y2="65" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        <line x1="26" y1="35" x2="44" y2="35" stroke="currentColor" stroke-width="2" stroke-linecap="round" opacity="0.5"/>
      </svg>

      <!-- Generic empty state -->
      <svg v-else width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="15" y="20" width="50" height="40" rx="4" stroke="currentColor" stroke-width="2" fill="none"/>
        <line x1="15" y1="35" x2="65" y2="35" stroke="currentColor" stroke-width="2" opacity="0.3"/>
        <circle cx="40" cy="50" r="8" stroke="currentColor" stroke-width="2" fill="none"/>
        <line x1="46" y1="56" x2="52" y2="62" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
    </div>

    <!-- Description text -->
    <p class="text-sm text-center max-w-xs">{{ description }}</p>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  /** Empty state type determines which illustration to show */
  type: 'screenshots' | 'dailyReport' | 'searchResults' | 'generic'
  /** Custom description text (optional, uses default if not provided) */
  description?: string
}>()
</script>
