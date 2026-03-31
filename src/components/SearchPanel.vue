<template>
  <BaseModal @close="$emit('close')" contentClass="w-[90vw] h-[90vh] max-w-4xl overflow-hidden flex flex-col">
    <!-- Header -->
    <div class="px-6 py-4 border-b border-[var(--color-border)] flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ t('searchPanel.title') }}</h2>
        <button @click="$emit('close')" class="text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]">✕</button>
      </div>

      <!-- Search Input -->
      <div class="px-6 py-4 border-b border-[var(--color-border)]">
        <div class="flex items-center gap-3">
          <div class="relative flex-1">
            <input
              type="text"
              v-model="searchQuery"
              @keyup.enter="search"
              :placeholder="t('searchPanel.placeholder')"
              class="w-full bg-[var(--color-surface-0)] border border-[var(--color-border-subtle)] rounded-lg px-4 py-2 text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] focus:border-primary focus:outline-none pr-10"
            />
            <button
              v-if="searchQuery"
              @click="clearSearch"
              class="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
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
          <span class="text-sm text-[var(--color-text-secondary)]">{{ t('searchPanel.sortBy') }}</span>
          <div class="flex items-center gap-2">
            <button
              @click="setOrderBy('rank')"
              :class="orderBy === 'rank' ? 'bg-primary text-white' : 'bg-[var(--color-surface-0)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'"
              class="px-3 py-1 rounded text-sm transition-colors"
            >
              {{ t('searchPanel.relevance') }}
            </button>
            <button
              @click="setOrderBy('time')"
              :class="orderBy === 'time' ? 'bg-primary text-white' : 'bg-[var(--color-surface-0)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'"
              class="px-3 py-1 rounded text-sm transition-colors"
            >
              {{ t('searchPanel.time') }}
            </button>
          </div>
          <span class="text-sm text-[var(--color-text-secondary)] ml-auto">
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
            class="absolute top-0 left-0 w-full py-3 px-2 hover:bg-[var(--color-surface-0)]/50 transition-colors border-b border-[var(--color-border)] cursor-pointer"
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
                    <span class="text-xs text-[var(--color-text-muted)]">{{ formatTime(results[virtualItem.index].record.timestamp) }}</span>
                    <span v-if="orderBy === 'rank'" class="text-xs text-[var(--color-text-muted)]">
                      {{ t('searchPanel.relevanceScore', { rank: results[virtualItem.index].rank.toFixed(2) }) }}
                    </span>
                  </div>
                  <p class="text-sm text-[var(--color-text-secondary)]" v-html="sanitizeSnippet(results[virtualItem.index].snippet)"></p>
                </div>
              </div>
            </template>
          </div>
        </div>

        <!-- Non-virtual scroll for small result sets -->
        <div v-else class="flex flex-col divide-y divide-[var(--color-border)]">
          <div
            v-for="result in results"
            :key="result.record.id"
            class="py-3 px-2 hover:bg-[var(--color-surface-0)]/50 transition-colors cursor-pointer"
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
                  <span class="text-xs text-[var(--color-text-muted)]">{{ formatTime(result.record.timestamp) }}</span>
                  <span v-if="orderBy === 'rank'" class="text-xs text-[var(--color-text-muted)]">
                    {{ t('searchPanel.relevanceScore', { rank: result.rank.toFixed(2) }) }}
                  </span>
                </div>
                <p class="text-sm text-[var(--color-text-secondary)]" v-html="sanitizeSnippet(result.snippet)"></p>
              </div>
            </div>
          </div>
        </div>
      </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useDebounceFn } from '@vueuse/core'
import BaseModal from './BaseModal.vue'
import EmptyState from './EmptyState.vue'
import { sanitizeSnippet } from '../utils/contentUtils'
import { showError } from '../stores/toast'
import type { Record } from '../types/tauri'
import SkeletonLoader from './SkeletonLoader.vue'

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
</script>

<style scoped>
:deep(mark) {
  background-color: #fef08a; /* yellow-300 */
  color: #1e293b; /* slate-800 */
  padding: 0 2px;
  border-radius: 2px;
}
</style>