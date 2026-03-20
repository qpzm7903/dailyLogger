<template>
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4">
    <div class="bg-dark rounded-xl w-full max-w-4xl max-h-[90vh] overflow-hidden flex flex-col border border-gray-700">
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-700">
        <div class="flex items-center gap-3">
          <span class="text-2xl">📈</span>
          <h2 class="font-medium text-lg">{{ t('timeline.title') }}</h2>
        </div>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white text-xl">
          ✕
        </button>
      </div>

      <!-- Date Selector -->
      <div class="px-5 py-3 border-b border-gray-700 flex items-center gap-4">
        <div class="flex items-center gap-2">
          <label class="text-sm text-gray-400">{{ t('timeline.date') }}:</label>
          <input
            type="date"
            v-model="selectedDate"
            class="bg-darker border border-gray-600 rounded-lg px-3 py-1.5 text-sm focus:outline-none focus:border-primary"
          />
        </div>
        <div class="flex gap-2">
          <button
            @click="goToPreviousDay"
            class="px-3 py-1.5 text-xs bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            ← {{ t('timeline.previousDay') }}
          </button>
          <button
            @click="goToToday"
            class="px-3 py-1.5 text-xs bg-primary hover:bg-blue-600 rounded-lg transition-colors"
          >
            {{ t('timeline.today') }}
          </button>
          <button
            @click="goToNextDay"
            class="px-3 py-1.5 text-xs bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            {{ t('timeline.nextDay') }} →
          </button>
        </div>
      </div>

      <!-- Stats Summary -->
      <div v-if="timelineData" class="px-5 py-3 bg-darker border-b border-gray-700">
        <div class="flex items-center gap-6 text-sm">
          <div class="flex items-center gap-2">
            <span class="text-gray-400">{{ t('timeline.totalEvents') }}:</span>
            <span class="text-white font-medium">{{ timelineData.total_events }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-gray-400">{{ t('timeline.activeHours') }}:</span>
            <span class="text-white font-medium">{{ timelineData.active_hours }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-gray-400">{{ t('timeline.workTimeEstimate') }}:</span>
            <span class="text-green-400 font-medium">{{ timelineData.work_time_estimate.toFixed(1) }}h</span>
          </div>
        </div>
      </div>

      <!-- Timeline Content -->
      <div class="flex-1 overflow-y-auto p-5">
        <div v-if="loading" class="text-center py-12 text-gray-500">
          {{ t('timeline.loading') }}
        </div>
        <div v-else-if="error" class="text-center py-12 text-red-400">
          {{ t('timeline.loadFailed', { error }) }}
        </div>
        <div v-else-if="!timelineData || timelineData.hour_groups.length === 0" class="text-center py-12 text-gray-500">
          {{ t('timeline.noEvents') }}
        </div>
        <div v-else class="space-y-3">
          <!-- Hour Groups -->
          <div
            v-for="group in timelineData.hour_groups"
            :key="group.hour"
            class="bg-darker rounded-lg border border-gray-700 overflow-hidden"
          >
            <!-- Hour Header -->
            <div
              @click="toggleHour(group.hour)"
              class="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-gray-800/50 transition-colors"
            >
              <div class="flex items-center gap-3">
                <span class="text-lg">{{ getHourIcon(group.hour) }}</span>
                <span class="text-sm font-medium">{{ group.label }}</span>
                <span class="text-xs text-gray-500">({{ group.count }} {{ t('timeline.events') }})</span>
              </div>
              <span class="text-gray-500 text-sm">
                {{ expandedHours.has(group.hour) ? '▼' : '▶' }}
              </span>
            </div>

            <!-- Events List -->
            <div
              v-show="expandedHours.has(group.hour)"
              class="border-t border-gray-700 divide-y divide-gray-700/50"
            >
              <div
                v-for="event in group.events"
                :key="event.record.id"
                @click="handleEventClick(event)"
                :class="[
                  'px-4 py-3 hover:bg-gray-800/30 transition-colors',
                  event.record.screenshot_path ? 'cursor-pointer' : 'cursor-default'
                ]"
              >
                <div class="flex items-center justify-between mb-1">
                  <div class="flex items-center gap-2">
                    <span class="text-xs text-gray-500">{{ event.time_str }}</span>
                    <span
                      :class="event.event_type === 'auto' ? 'text-blue-400' : 'text-green-400'"
                      class="text-xs"
                    >
                      {{ event.event_type === 'auto' ? '🖥️' : '⚡' }}
                    </span>
                  </div>
                  <span v-if="event.record.screenshot_path" class="text-xs text-primary">
                    📷
                  </span>
                </div>
                <p class="text-sm text-gray-300">{{ event.preview }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import type { LogRecord, Tag } from '../types/tauri';

interface TimelineEvent {
  record: LogRecord
  time_str: string
  event_type: 'auto' | 'manual'
  preview: string
}

interface HourGroup {
  hour: number
  label: string
  count: number
  events: TimelineEvent[]
}

interface TimelineData {
  total_events: number
  active_hours: number
  work_time_estimate: number
  hour_groups: HourGroup[]
}

const { t } = useI18n();

const props = defineProps<{
  initialDate?: string | null
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'viewScreenshot', record: LogRecord): void;
}>();

const selectedDate = ref('');
const timelineData = ref<TimelineData | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const expandedHours = ref<Set<number>>(new Set());

// Initialize with today's date or provided initial date
onMounted(() => {
  if (props.initialDate) {
    selectedDate.value = props.initialDate;
  } else {
    selectedDate.value = new Date().toISOString().split('T')[0];
  }
  loadTimelineData();
});

// Watch for date changes
watch(selectedDate, () => {
  loadTimelineData();
});

async function loadTimelineData() {
  if (!selectedDate.value) return;

  loading.value = true;
  error.value = null;

  try {
    const result = await invoke<TimelineData>('get_timeline_for_date', { date: selectedDate.value });
    timelineData.value = result;
    // Auto-expand all hours with events
    expandedHours.value = new Set(result.hour_groups.map(g => g.hour));
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function toggleHour(hour: number) {
  const newSet = new Set(expandedHours.value);
  if (newSet.has(hour)) {
    newSet.delete(hour);
  } else {
    newSet.add(hour);
  }
  expandedHours.value = newSet;
}

function getHourIcon(hour: number) {
  if (hour >= 6 && hour < 12) return '🌅';
  if (hour >= 12 && hour < 14) return '☀️';
  if (hour >= 14 && hour < 18) return '🌤️';
  if (hour >= 18 && hour < 22) return '🌆';
  return '🌙';
}

function goToPreviousDay() {
  const current = new Date(selectedDate.value);
  current.setDate(current.getDate() - 1);
  selectedDate.value = current.toISOString().split('T')[0];
}

function goToNextDay() {
  const current = new Date(selectedDate.value);
  current.setDate(current.getDate() + 1);
  selectedDate.value = current.toISOString().split('T')[0];
}

function goToToday() {
  selectedDate.value = new Date().toISOString().split('T')[0];
}

function handleEventClick(event: TimelineEvent) {
  if (event.record.screenshot_path) {
    emit('viewScreenshot', event.record);
  }
}
</script>