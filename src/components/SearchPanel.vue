<template>
  <div
    class="fixed inset-0 bg-black/80 flex items-center justify-center z-50"
    @click.self="$emit('close')"
    :ref="focusTrap.containerRef"
  >
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
      <div ref="scrollContainer" class="flex-1 overflow-auto p-4">
        <!-- Loading skeleton -->
        <div v-if="isLoading">
          <SkeletonLoader :count="5" />
        </div>
        <!-- Empty state: no results -->
        <EmptyState v-else-if="hasSearched && results.length === 0" type="searchResults" :description="t('emptyState.searchResults')" />
        <!-- Empty state: not searched yet -->
        <EmptyState v-else-if="!hasSearched" type="generic" :description="t('searchPanel.startHint')" />

        <!-- UX-022: Virtual scroll for large result sets -->
        <div
          v-else-if="shouldUseVirtualScroll"
          class="relative"
          :style="{ height: `${virtualizer.getTotalSize()}px` }"
        >
          <div
            v-for="virtualItem in virtualItems"
            :key="virtualItem.index"
            class="absolute top-0 left-0 w-full py-3 px-2 hover:bg-darker/50 transition-colors border-b border-gray-700 cursor-pointer"
            :style="{
              height: `${virtualItem.size}px`,
              transform: `translateY(${virtualItem.start}px)`,
            }"
            :data-index="virtualItem.index"
            @click="results[virtualItem.index] && handleResultClick(results[virtualItem.index].record)"
          >
            <template v-if="results[virtualItem.index]">
              <div class="flex items-start justify-between gap-2">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <span
                      :class="results[virtualItem.index].record.source_type === 'auto' ? 'bg-blue-500/20 text-blue-400' : 'bg-green-500/20 text-green-400'"
                      class="px-2 py-0.5 rounded text-xs"
                    >
                      {{ results[virtualItem.index].record.source_type === 'auto' ? t('searchPanel.auto') : t('searchPanel.manual') }}
                    </span>
                    <span class="text-xs text-gray-500">{{ formatTime(results[virtualItem.index].record.timestamp) }}</span>
                    <span v-if="orderBy === 'rank'" class="text-xs text-gray-600">
                      {{ t('searchPanel.relevanceScore', { rank: results[virtualItem.index].rank.toFixed(2) }) }}
                    </span>
                  </div>
                  <p class="text-sm text-gray-300" v-html="results[virtualItem.index].snippet"></p>
                </div>
              </div>
            </template>
          </div>
        </div>

        <!-- Non-virtual scroll for small result sets -->
        <div v-else class="flex flex-col divide-y divide-gray-700">
          <div
            v-for="result in results"
            :key="result.record.id"
            class="py-3 px-2 hover:bg-darker/50 transition-colors cursor-pointer"
            @click="handleResultClick(result.record)"
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
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useDebounceFn } from '@vueuse/core'
import EmptyState from './EmptyState.vue'
import { useModal } from '../composables/useModal'
import { showError } from '../stores/toast'
import SkeletonLoader from './SkeletonLoader.vue'
import type { Record } from '../types/tauri'

interface SearchResult {
  record: Record
  snippet: string
  rank: number
}

const { t } = useI18n()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'viewScreenshot', record: Record): void
}>()
const { focusTrap } = useModal()

// UX-022: Virtual scroll configuration
const VIRTUAL_SCROLL_CONFIG = {
  itemHeight: 72,          // Fixed height per result (px)
  overscan: 5,             // Render extra items outside viewport
  threshold: 50            // Enable virtual scroll when results exceed this
}

// State
const searchQuery = ref('')
const results = ref<SearchResult[]>([])
const isLoading = ref(false)
const hasSearched = ref(false)
const orderBy = ref<'rank' | 'time'>('rank')
const scrollContainer = ref<HTMLElement | null>(null)

// UX-022: Virtual scroll - only enable for large result sets
const shouldUseVirtualScroll = computed(() => results.value.length > VIRTUAL_SCROLL_CONFIG.threshold)

// UX-022: Virtualizer instance
const virtualizer = useVirtualizer({
  count: results.value.length,
  getScrollElement: () => scrollContainer.value,
  estimateSize: () => VIRTUAL_SCROLL_CONFIG.itemHeight,
  overscan: VIRTUAL_SCROLL_CONFIG.overscan,
})

// UX-022: Virtual items to render
const virtualItems = computed(() => virtualizer.value.getVirtualItems())

// AC 6: Debounce search (>=300ms)
const debouncedSearch = useDebounceFn(async () => {
  await search()
}, 300)

// Watch for query changes and trigger debounced search
watch(searchQuery, (newQuery) => {
  if (newQuery.trim() && hasSearched.value) {
    debouncedSearch()
  }
})

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
      limit: 200  // Increased limit for virtual scroll demo
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

// AC 4: Handle result click - emit event to parent for screenshot viewing
function handleResultClick(record: Record) {
  emit('viewScreenshot', record)
  emit('close')
}

// UX-5: Focus trap lifecycle
onMounted(() => {
  focusTrap.activate()
})

onBeforeUnmount(() => {
  focusTrap.deactivate()
})
</script>

<style scoped>
:deep(mark) {
  background-color: #fef08a; /* yellow-300 */
  color: #1e293b; /* slate-800 */
  padding: 0 2px;
  border-radius: 2px;
}
</style>