<template>
  <div
    class="bg-[var(--color-surface-1)]/60 backdrop-blur-md rounded-2xl p-5 border border-[var(--color-border)]/50 shadow-xl transition-all duration-200 hover:shadow-2xl"
  >
    <!-- Header -->
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-2">
        <span class="text-2xl">📈</span>
        <h2 class="font-medium text-[var(--color-text-primary)]">{{ t('timelineWidget.title') }}</h2>
      </div>
      <button
        @click="$emit('openFullTimeline')"
        class="text-xs text-primary hover:underline flex items-center gap-1"
      >
        {{ t('timelineWidget.viewFull') }}
        <span class="text-sm">→</span>
      </button>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-4 text-[var(--color-text-muted)] text-sm">
      {{ t('timelineWidget.loading') }}
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="text-center py-4 text-red-400 text-sm">
      {{ t('timelineWidget.loadFailed') }}
    </div>

    <!-- Timeline Heatmap -->
    <div v-else class="space-y-4">
      <!-- 24-hour Heatmap -->
      <div class="grid grid-cols-12 gap-1">
        <div
          v-for="hour in 24"
          :key="hour - 1"
          @mouseenter="hoveredHour = hour - 1"
          @mouseleave="hoveredHour = null"
          @click="handleHourClick(hour - 1)"
          :class="getHourCellClass(hour - 1)"
          class="h-6 rounded-sm cursor-pointer transition-all duration-150 relative"
          :title="getHourTooltip(hour - 1)"
        >
          <!-- Tooltip -->
          <div
            v-if="hoveredHour === hour - 1"
            class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 bg-gray-900 text-white text-xs rounded shadow-lg whitespace-nowrap z-10"
          >
            {{ getHourTooltip(hour - 1) }}
          </div>
        </div>
      </div>

      <!-- Hour Labels -->
      <div class="flex justify-between text-xs text-[var(--color-text-muted)] px-1">
        <span>0</span>
        <span>6</span>
        <span>12</span>
        <span>18</span>
        <span>24</span>
      </div>

      <!-- Stats Summary -->
      <div class="flex items-center gap-4 pt-2 border-t border-[var(--color-border)]/50">
        <div class="flex items-center gap-1.5">
          <span class="text-[var(--color-text-secondary)] text-xs">{{ t('timelineWidget.workTime') }}:</span>
          <span class="text-green-400 font-medium text-sm">
            {{ timelineData?.work_time_estimate?.toFixed(1) ?? '0.0' }}h
          </span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="text-[var(--color-text-secondary)] text-xs">{{ t('timelineWidget.activePeriods') }}:</span>
          <span class="text-[var(--color-text-primary)] font-medium text-sm">
            {{ timelineData?.active_hours ?? 0 }}
          </span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="text-[var(--color-text-secondary)] text-xs">{{ t('timelineWidget.totalRecords') }}:</span>
          <span class="text-[var(--color-text-primary)] font-medium text-sm">
            {{ timelineData?.total_events ?? 0 }}
          </span>
        </div>
      </div>

      <!-- Legend -->
      <div class="flex items-center gap-4 text-xs text-[var(--color-text-muted)]">
        <div class="flex items-center gap-1">
          <span class="w-3 h-3 rounded-sm bg-blue-500"></span>
          <span>{{ t('timelineWidget.autoCapture') }}</span>
        </div>
        <div class="flex items-center gap-1">
          <span class="w-3 h-3 rounded-sm bg-green-500"></span>
          <span>{{ t('timelineWidget.manualNote') }}</span>
        </div>
        <div class="flex items-center gap-1">
          <span class="w-3 h-3 rounded-sm bg-[var(--color-surface-2)]/30"></span>
          <span>{{ t('timelineWidget.noActivity') }}</span>
        </div>
      </div>
    </div>

    <!-- Expanded Hour Details -->
    <div
      v-if="expandedHour !== null && getHourEvents(expandedHour).length > 0"
      class="mt-4 pt-4 border-t border-[var(--color-border)]/50"
    >
      <div class="flex items-center justify-between mb-2">
        <h3 class="text-sm font-medium text-[var(--color-text-secondary)]">
          {{ formatHourLabel(expandedHour) }} ({{ getHourEvents(expandedHour).length }} {{ t('timelineWidget.events') }})
        </h3>
        <button
          @click="expandedHour = null"
          class="text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] text-sm"
        >
          ✕
        </button>
      </div>
      <div class="space-y-2 max-h-32 overflow-y-auto">
        <div
          v-for="event in getHourEvents(expandedHour)"
          :key="event.record.id"
          class="bg-[var(--color-surface-0)]/50 rounded-lg p-2 text-sm"
        >
          <div class="flex items-center justify-between">
            <span class="text-[var(--color-text-muted)] text-xs">{{ event.time_str }}</span>
            <span :class="event.event_type === 'auto' ? 'text-blue-400' : 'text-green-400'" class="text-xs">
              {{ event.event_type === 'auto' ? '🖥️' : '⚡' }}
            </span>
          </div>
          <p class="text-[var(--color-text-secondary)] text-xs mt-1 truncate">{{ event.preview }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import type { LogRecord } from '../types/tauri';

interface TimelineEvent {
  record: LogRecord;
  time_str: string;
  event_type: 'auto' | 'manual';
  preview: string;
}

interface HourGroup {
  hour: number;
  label: string;
  count: number;
  events: TimelineEvent[];
}

interface TimelineData {
  date: string;
  hour_groups: HourGroup[];
  total_events: number;
  active_hours: number;
  work_time_estimate: number;
}

const { t } = useI18n();

const emit = defineEmits<{
  (e: 'openFullTimeline'): void;
  (e: 'refresh'): void;
}>();

const timelineData = ref<TimelineData | null>(null);
const loading = ref(true); // Start as loading since we call loadTimelineData on mount
const error = ref<string | null>(null);
const hoveredHour = ref<number | null>(null);
const expandedHour = ref<number | null>(null);

// Map hour groups for quick lookup
const hourMap = computed(() => {
  const map = new Map<number, HourGroup>();
  if (timelineData.value && Array.isArray(timelineData.value.hour_groups)) {
    for (const group of timelineData.value.hour_groups) {
      map.set(group.hour, group);
    }
  }
  return map;
});

// Get events for a specific hour
function getHourEvents(hour: number): TimelineEvent[] {
  return hourMap.value.get(hour)?.events ?? [];
}

// Determine cell class based on hour activity
function getHourCellClass(hour: number): string {
  const group = hourMap.value.get(hour);

  if (!group || group.count === 0) {
    return 'bg-[var(--color-surface-2)]/30 hover:bg-[var(--color-surface-2)]/50';
  }

  // Check event types in this hour
  const hasAuto = group.events.some(e => e.event_type === 'auto');
  const hasManual = group.events.some(e => e.event_type === 'manual');

  // Determine intensity based on event count
  const intensity = Math.min(Math.ceil(group.count / 2), 4);

  if (hasAuto && hasManual) {
    // Mixed: purple gradient
    return `bg-purple-${400 + intensity * 50} hover:bg-purple-${500 + intensity * 50}`;
  } else if (hasManual) {
    // Manual notes: green
    return `bg-green-${400 + intensity * 50} hover:bg-green-${500 + intensity * 50}`;
  } else {
    // Auto screenshots: blue
    return `bg-blue-${400 + intensity * 50} hover:bg-blue-${500 + intensity * 50}`;
  }
}

// Get tooltip for hour cell
function getHourTooltip(hour: number): string {
  const group = hourMap.value.get(hour);

  if (!group || group.count === 0) {
    return `${hour.toString().padStart(2, '0')}:00 - ${(hour + 1).toString().padStart(2, '0')}:00: ${t('timelineWidget.noEvents')}`;
  }

  const autoCount = group.events.filter(e => e.event_type === 'auto').length;
  const manualCount = group.events.filter(e => e.event_type === 'manual').length;

  const parts = [];
  if (autoCount > 0) parts.push(`${autoCount} ${t('timelineWidget.auto')}`);
  if (manualCount > 0) parts.push(`${manualCount} ${t('timelineWidget.manual')}`);

  return `${hour.toString().padStart(2, '0')}:00 - ${(hour + 1).toString().padStart(2, '0')}:00: ${parts.join(', ')}`;
}

// Format hour label
function formatHourLabel(hour: number): string {
  return `${hour.toString().padStart(2, '0')}:00 - ${(hour + 1).toString().padStart(2, '0')}:00`;
}

// Handle hour click
function handleHourClick(hour: number) {
  const events = getHourEvents(hour);
  if (events.length > 0) {
    expandedHour.value = expandedHour.value === hour ? null : hour;
  }
}

// Load timeline data
async function loadTimelineData() {
  loading.value = true;
  error.value = null;

  try {
    const result = await invoke<TimelineData>('get_timeline_today');
    timelineData.value = result;
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

// Refresh data
async function refresh() {
  await loadTimelineData();
}

// Initial load
onMounted(() => {
  loadTimelineData();
});

// Expose refresh method for parent components
defineExpose({ refresh });
</script>