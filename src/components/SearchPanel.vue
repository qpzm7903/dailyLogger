<template>
  <div class="fixed inset-0 bg-black/80 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[90vw] h-[90vh] max-w-4xl overflow-hidden border border-gray-700 flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('searchPanel.title') }}</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <!-- Search Input -->
      <div class="px-6 py-4 border-b border-gray-700">
        <div class="flex items-center gap-3">
          <div class="relative flex-1">
            <input
              type="text"
              v-model="searchQuery"
              @keyup.enter="search"
              :placeholder="t('searchPanel.placeholder')"
              class="w-full bg-darker border border-gray-600 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:border-primary focus:outline-none pr-10"
            />
            <button
              v-if="searchQuery"
              @click="clearSearch"
              class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-white"
            >
              ✕
            </button>
          </div>
          <button
            @click="search"
            :disabled="isLoading || !searchQuery.trim()"
            class="px-6 py-2 bg-primary text-white rounded-lg hover:bg-primary/80 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {{ isLoading ? t('searchPanel.searching') : t('searchPanel.search') }}
          </button>
        </div>

        <!-- Sort Toggle -->
        <div v-if="results.length > 0" class="flex items-center gap-4 mt-3">
          <span class="text-sm text-gray-400">{{ t('searchPanel.sortBy') }}</span>
          <div class="flex items-center gap-2">
            <button
              @click="setOrderBy('rank')"
              :class="orderBy === 'rank' ? 'bg-primary text-white' : 'bg-darker text-gray-400 hover:text-white'"
              class="px-3 py-1 rounded text-sm transition-colors"
            >
              {{ t('searchPanel.relevance') }}
            </button>
            <button
              @click="setOrderBy('time')"
              :class="orderBy === 'time' ? 'bg-primary text-white' : 'bg-darker text-gray-400 hover:text-white'"
              class="px-3 py-1 rounded text-sm transition-colors"
            >
              {{ t('searchPanel.time') }}
            </button>
          </div>
          <span class="text-sm text-gray-400 ml-auto">
            {{ t('searchPanel.totalResults', { count: results.length }) }}
          </span>
        </div>
      </div>

      <!-- Results List -->
      <div class="flex-1 overflow-auto p-4">
        <div v-if="isLoading" class="text-center py-8 text-gray-500">
          {{ t('searchPanel.searching') }}
        </div>
        <div v-else-if="hasSearched && results.length === 0" class="text-center py-8 text-gray-500">
          {{ t('searchPanel.noResults') }}
        </div>
        <div v-else-if="!hasSearched" class="text-center py-8 text-gray-500">
          {{ t('searchPanel.startHint') }}
        </div>

        <div v-else class="flex flex-col divide-y divide-gray-700">
          <div
            v-for="result in results"
            :key="result.record.id"
            class="py-3 px-2 hover:bg-darker/50 transition-colors"
          >
            <div class="flex items-start justify-between gap-2">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <span
                    :class="result.record.source_type === 'auto' ? 'bg-blue-500/20 text-blue-400' : 'bg-green-500/20 text-green-400'"
                    class="px-2 py-0.5 rounded text-xs"
                  >
                    {{ result.record.source_type === 'auto' ? t('searchPanel.auto') : t('searchPanel.manual') }}
                  </span>
                  <span class="text-xs text-gray-500">{{ formatTime(result.record.timestamp) }}</span>
                  <span v-if="orderBy === 'rank'" class="text-xs text-gray-600">
                    {{ t('searchPanel.relevanceScore', { rank: result.rank.toFixed(2) }) }}
                  </span>
                </div>
                <p class="text-sm text-gray-300" v-html="result.snippet"></p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { showError } from '../stores/toast'
import type { Record } from '../types/tauri'

interface SearchResult {
  record: Record
  snippet: string
  rank: number
}

const { t } = useI18n()
const emit = defineEmits<{(e: 'close'): void}>()

// State
const searchQuery = ref('')
const results = ref<SearchResult[]>([])
const isLoading = ref(false)
const hasSearched = ref(false)
const orderBy = ref<'rank' | 'time'>('rank')

function formatTime(timestamp: string) {
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function clearSearch() {
  searchQuery.value = ''
  results.value = []
  hasSearched.value = false
}

async function search() {
  if (!searchQuery.value.trim()) return

  isLoading.value = true
  hasSearched.value = true

  try {
    const searchResults = await invoke<SearchResult[]>('search_records', {
      query: searchQuery.value.trim(),
      orderBy: orderBy.value,
      limit: 50
    })
    results.value = searchResults
  } catch (error) {
    showError(t('searchPanel.searchFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

async function setOrderBy(newOrderBy: 'rank' | 'time') {
  if (orderBy.value === newOrderBy) return
  orderBy.value = newOrderBy

  // Re-search with new order if we have a query
  if (searchQuery.value.trim() && hasSearched.value) {
    await search()
  }
}
</script>

<style scoped>
:deep(mark) {
  background-color: #fef08a; /* yellow-300 */
  color: #1e293b; /* slate-800 */
  padding: 0 2px;
  border-radius: 2px;
}
</style>