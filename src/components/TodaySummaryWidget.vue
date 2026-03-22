<template>
  <div
    class="bg-dark/60 backdrop-blur-md rounded-2xl p-5 border border-gray-700/50 shadow-xl transition-all duration-200 hover:shadow-2xl"
  >
    <!-- Header with collapse toggle -->
    <div
      class="flex items-center justify-between cursor-pointer"
      @click="toggleCollapsed"
    >
      <div class="flex items-center gap-2">
        <span class="text-2xl">📊</span>
        <h2 class="font-medium text-white">{{ t('widget.todaySummary') }}</h2>
      </div>
      <span class="text-gray-400 text-xs transition-transform duration-200" :class="isCollapsed ? '' : 'rotate-180'">
        ▼
      </span>
    </div>

    <!-- Expandable content -->
    <Transition name="slide">
      <div v-if="!isCollapsed" class="mt-4">
        <!-- Loading State -->
        <div v-if="loading" class="text-center py-4 text-gray-500 text-sm">
          {{ t('widget.loading') }}
        </div>

        <!-- Error State -->
        <div v-else-if="error" class="text-center py-4 text-red-400 text-sm">
          {{ t('widget.loadFailed') }}
        </div>

        <!-- Empty State -->
        <div v-else-if="stats && stats.total_count === 0" class="text-center py-4 text-gray-400 text-sm">
          {{ t('widget.noRecordsYet') }}
        </div>

        <!-- Stats Content -->
        <div v-else-if="stats" class="space-y-4">
          <!-- Record Counts -->
          <div class="grid grid-cols-3 gap-3 text-center">
            <div class="bg-darker/50 rounded-xl p-3">
              <div class="text-2xl font-bold text-primary">{{ stats.total_count }}</div>
              <div class="text-xs text-gray-400">{{ t('widget.totalRecords') }}</div>
            </div>
            <div class="bg-darker/50 rounded-xl p-3">
              <div class="text-lg font-semibold text-blue-400">{{ stats.auto_count }}</div>
              <div class="text-xs text-gray-400">{{ t('widget.autoCaptures') }}</div>
            </div>
            <div class="bg-darker/50 rounded-xl p-3">
              <div class="text-lg font-semibold text-green-400">{{ stats.manual_count }}</div>
              <div class="text-xs text-gray-400">{{ t('widget.manualNotes') }}</div>
            </div>
          </div>

          <!-- Time Range and Busiest Hour -->
          <div v-if="stats.total_count > 0" class="text-xs text-gray-400 space-y-1.5 pt-2 border-t border-gray-700/50">
            <div v-if="stats.first_record_time && stats.latest_record_time" class="flex items-center gap-2">
              <span>⏱</span>
              <span>{{ formatTime(stats.first_record_time) }} – {{ formatTime(stats.latest_record_time) }}</span>
            </div>
            <div v-if="stats.busiest_hour !== null && stats.busiest_hour !== undefined" class="flex items-center gap-2">
              <span>🔥</span>
              <span>
                {{ t('widget.busiestHour') }}: {{ stats.busiest_hour.toString().padStart(2, '0') }}:00
                ({{ stats.busiest_hour_count }} {{ t('widget.records') }})
              </span>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';

// TodayStats interface matching the Rust struct
interface TodayStats {
  total_count: number;
  auto_count: number;
  manual_count: number;
  first_record_time: string | null;
  latest_record_time: string | null;
  busiest_hour: number | null;
  busiest_hour_count: number;
}

const { t } = useI18n();

const STORAGE_KEY = 'today-widget-collapsed';

const stats = ref<TodayStats | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);
const isCollapsed = ref(localStorage.getItem(STORAGE_KEY) === 'true');

// Toggle collapsed state and persist to localStorage
function toggleCollapsed() {
  isCollapsed.value = !isCollapsed.value;
  localStorage.setItem(STORAGE_KEY, String(isCollapsed.value));
}

// Format RFC3339 timestamp to HH:MM
function formatTime(rfc3339: string): string {
  try {
    const date = new Date(rfc3339);
    return date.toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: false
    });
  } catch {
    return rfc3339;
  }
}

// Load stats from backend
async function loadStats() {
  loading.value = true;
  error.value = null;

  try {
    const result = await invoke<TodayStats>('get_today_stats');
    stats.value = result;
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

// Refresh data (exposed for parent components)
async function refresh() {
  await loadStats();
}

// Initial load
onMounted(() => {
  loadStats();
});

// Expose refresh method for parent components
defineExpose({ refresh });
</script>

<style scoped>
.slide-enter-active,
.slide-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  max-height: 0;
  margin-top: 0;
}

.slide-enter-to,
.slide-leave-from {
  opacity: 1;
  max-height: 500px;
}
</style>