<template>
  <div class="space-y-6">
    <!-- Capture Only Mode -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.captureOnlyMode') }}</h3>
      <div class="space-y-3">
        <div class="flex items-center gap-2">
          <input
            v-model="localSettings.capture_only_mode"
            type="checkbox"
            id="capture_only_mode"
            class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
          />
          <label for="capture_only_mode" class="text-xs text-gray-300 cursor-pointer">
            {{ $t('settings.captureOnlyModeLabel') }}
          </label>
        </div>
        <span class="text-xs text-gray-500 block">
          {{ $t('settings.captureOnlyModeHint') }}
        </span>
      </div>
    </div>

    <!-- Time Strategy -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.timeStrategy') }}</h3>
      <div class="space-y-3">
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.screenshotInterval') }}</label>
          <input
            v-model.number="localSettings.screenshot_interval"
            type="number"
            min="1"
            max="60"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
          />
        </div>
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.summaryTime') }}</label>
          <input
            v-model="localSettings.summary_time"
            type="time"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
          />
        </div>
      </div>
    </div>

    <!-- Smart Deduplication -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.smartDedup') }}</h3>
      <div class="space-y-3">
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.changeThreshold') }}</label>
          <input
            v-model.number="localSettings.change_threshold"
            type="number"
            min="1"
            max="20"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
          />
          <span class="text-xs text-gray-500 mt-1 block">{{ $t('settings.changeThresholdHint') }}</span>
        </div>
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.maxSilentTime') }}</label>
          <input
            v-model.number="localSettings.max_silent_minutes"
            type="number"
            min="5"
            max="120"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
          />
          <span class="text-xs text-gray-500 mt-1 block">{{ $t('settings.maxSilentTimeHint') }}</span>
        </div>
      </div>
    </div>

    <!-- Silent Threshold Adjustment -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.silentThresholdAdjust') }}</h3>
      <div class="space-y-3">
        <div class="flex items-center gap-2">
          <input
            v-model="localSettings.auto_adjust_silent"
            type="checkbox"
            id="auto_adjust_silent"
            class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
          />
          <label for="auto_adjust_silent" class="text-xs text-gray-300 cursor-pointer">
            {{ $t('settings.autoAdjustSilent') }}
          </label>
        </div>
        <span class="text-xs text-gray-500 block">
          {{ $t('settings.autoAdjustHint') }}
        </span>
      </div>
    </div>

    <!-- Work Time Detection -->
    <div>
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.workTimeDetection') }}</h3>
      <div class="space-y-3">
        <div class="flex items-center gap-2">
          <input
            v-model="localSettings.auto_detect_work_time"
            type="checkbox"
            id="auto_detect_work_time"
            class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
          />
          <label for="auto_detect_work_time" class="text-xs text-gray-300 cursor-pointer">
            {{ $t('settings.autoDetectWorkTime') }}
          </label>
        </div>
        <span class="text-xs text-gray-500 block">
          {{ $t('settings.autoDetectHint') }}
        </span>

        <!-- Custom work time toggle -->
        <div class="flex items-center gap-2 pt-2" v-if="localSettings.auto_detect_work_time">
          <input
            v-model="localSettings.use_custom_work_time"
            type="checkbox"
            id="use_custom_work_time"
            class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
          />
          <label for="use_custom_work_time" class="text-xs text-gray-300 cursor-pointer">
            {{ $t('settings.useCustomWorkTime') }}
          </label>
        </div>

        <!-- Custom work time inputs -->
        <div v-if="localSettings.auto_detect_work_time && localSettings.use_custom_work_time" class="grid grid-cols-2 gap-3 pt-2">
          <div>
            <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.startTime') }}</label>
            <input
              v-model="localSettings.custom_work_time_start"
              type="time"
              class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
            />
          </div>
          <div>
            <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.endTime') }}</label>
            <input
              v-model="localSettings.custom_work_time_end"
              type="time"
              class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Window Filter -->
    <div v-if="isDesktop">
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.windowFilter') }}</h3>
      <div class="space-y-3">
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.windowWhitelist') }}</label>
          <div class="flex flex-wrap gap-2 mb-2">
            <span
              v-for="(tag, index) in whitelistTags"
              :key="'wl-' + index"
              class="inline-flex items-center gap-1 px-2 py-1 bg-primary/20 text-primary text-xs rounded-lg"
            >
              {{ tag }}
              <button
                type="button"
                @click="removeWhitelistTag(index)"
                class="hover:text-white transition-colors"
              >✕</button>
            </span>
          </div>
          <input
            v-model="newWhitelistTag"
            type="text"
            :placeholder="$t('settings.whitelistPlaceholder')"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
            @keyup.enter="addWhitelistTag"
          />
        </div>
        <div>
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.windowBlacklist') }}</label>
          <div class="flex flex-wrap gap-2 mb-2">
            <span
              v-for="(tag, index) in blacklistTags"
              :key="'bl-' + index"
              class="inline-flex items-center gap-1 px-2 py-1 bg-red-500/20 text-red-400 text-xs rounded-lg"
            >
              {{ tag }}
              <button
                type="button"
                @click="removeBlacklistTag(index)"
                class="hover:text-white transition-colors"
              >✕</button>
            </span>
          </div>
          <input
            v-model="newBlacklistTag"
            type="text"
            :placeholder="$t('settings.blacklistPlaceholder')"
            class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
            @keyup.enter="addBlacklistTag"
          />
        </div>
        <div class="flex items-center gap-2 pt-1">
          <input
            v-model="localSettings.use_whitelist_only"
            type="checkbox"
            id="use_whitelist_only"
            class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
          />
          <label for="use_whitelist_only" class="text-xs text-gray-300 cursor-pointer">
            {{ $t('settings.whitelistOnly') }}
          </label>
        </div>
      </div>
    </div>

    <!-- Display Settings -->
    <div v-if="isDesktop">
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.displaySettings') }}</h3>
      <div class="space-y-3">
        <!-- Multi-monitor capture mode -->
        <div v-if="(monitors?.length ?? 0) > 1" class="space-y-2">
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.captureMode') }}</label>
          <div class="flex flex-wrap gap-4">
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                v-model="localSettings.capture_mode"
                value="primary"
                class="w-4 h-4 border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0"
              />
              <span class="text-sm text-gray-300">{{ $t('settings.primaryMonitor') }}</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                v-model="localSettings.capture_mode"
                value="secondary"
                class="w-4 h-4 border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0"
              />
              <span class="text-sm text-gray-300">{{ $t('settings.secondaryMonitor') }}</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                v-model="localSettings.capture_mode"
                value="all"
                class="w-4 h-4 border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0"
              />
              <span class="text-sm text-gray-300">{{ $t('settings.allMonitors') }}</span>
            </label>
          </div>
        </div>

        <!-- Monitor list -->
        <div v-if="(monitors?.length ?? 0) > 1" class="space-y-1">
          <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.connectedDisplays') }}</label>
          <div
            v-for="m in monitors"
            :key="m.index"
            class="flex items-center gap-2 text-sm bg-darker rounded-lg px-3 py-2 border border-gray-700"
          >
            <span class="text-gray-300">{{ m.name }}</span>
            <span class="text-gray-500">{{ m.resolution }}</span>
            <span v-if="m.is_primary" class="text-xs bg-primary/20 text-primary px-1.5 py-0.5 rounded">{{ $t('settings.primary') }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { usePlatform } from '../../composables/usePlatform'

// Props
interface Monitor {
  id: number
  name: string
  width: number
  height: number
  is_primary: boolean
  index?: number
  resolution?: string
}

interface Props {
  settings: {
    screenshot_interval: number
    summary_time: string
    change_threshold: number
    max_silent_minutes: number
    auto_adjust_silent: boolean
    auto_detect_work_time: boolean
    use_custom_work_time: boolean
    custom_work_time_start: string
    custom_work_time_end: string
    use_whitelist_only: boolean
    capture_mode: string
    selected_monitor_index: number
    capture_only_mode: boolean
  }
  whitelistTags: string[]
  blacklistTags: string[]
  monitors?: Monitor[]
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  (e: 'update:settings', value: Props['settings']): void
  (e: 'update:whitelistTags', value: string[]): void
  (e: 'update:blacklistTags', value: string[]): void
}>()

// Composables
const { isDesktop } = usePlatform()

// Local state
const localSettings = ref({ ...props.settings })
const localWhitelistTags = ref([...props.whitelistTags])
const localBlacklistTags = ref([...props.blacklistTags])

// Tag input state
const newWhitelistTag = ref('')
const newBlacklistTag = ref('')

// Watch for external changes
watch(() => props.settings, (newVal) => {
  localSettings.value = { ...newVal }
}, { deep: true })

watch(() => props.whitelistTags, (newVal) => {
  localWhitelistTags.value = [...newVal]
}, { deep: true })

watch(() => props.blacklistTags, (newVal) => {
  localBlacklistTags.value = [...newVal]
}, { deep: true })

// Watch for local changes and emit
watch(localSettings, (newVal) => {
  emit('update:settings', newVal)
}, { deep: true })

// Tag management methods
function addWhitelistTag() {
  if (newWhitelistTag.value.trim()) {
    localWhitelistTags.value.push(newWhitelistTag.value.trim())
    emit('update:whitelistTags', [...localWhitelistTags.value])
    newWhitelistTag.value = ''
  }
}

function removeWhitelistTag(index: number) {
  localWhitelistTags.value.splice(index, 1)
  emit('update:whitelistTags', [...localWhitelistTags.value])
}

function addBlacklistTag() {
  if (newBlacklistTag.value.trim()) {
    localBlacklistTags.value.push(newBlacklistTag.value.trim())
    emit('update:blacklistTags', [...localBlacklistTags.value])
    newBlacklistTag.value = ''
  }
}

function removeBlacklistTag(index: number) {
  localBlacklistTags.value.splice(index, 1)
  emit('update:blacklistTags', [...localBlacklistTags.value])
}
</script>