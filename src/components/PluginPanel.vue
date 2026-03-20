<template>
  <div class="space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-gray-300 mb-3">{{ t('plugin.title') }}</h3>
      <button
        @click="openPluginsDir"
        class="px-3 py-1 text-sm bg-primary text-darker rounded hover:bg-opacity-80 transition"
      >
        {{ t('plugin.openDirectory') }}
      </button>
    </div>

    <!-- Plugin List -->
    <div v-if="loading" class="text-center py-4 text-gray-400">
      {{ t('plugin.loading') }}
    </div>

    <div v-else-if="plugins.length === 0" class="text-center py-4 text-gray-400">
      {{ t('plugin.noPlugins') }}
    </div>

    <div v-else class="space-y-3">
      <div
        v-for="plugin in plugins"
        :key="plugin.id"
        class="bg-darker rounded-lg p-4 border border-gray-700"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1">
            <div class="flex items-center gap-2">
              <h4 class="text-primary font-medium">{{ plugin.name }}</h4>
              <span
                class="text-xs px-2 py-0.5 rounded"
                :class="{
                  'bg-green-900 text-green-300': plugin.status === 'ready' && plugin.enabled,
                  'bg-yellow-900 text-yellow-300': plugin.status === 'disabled' || !plugin.enabled,
                  'bg-red-900 text-red-300': plugin.status === 'error'
                }"
              >
                {{ getStatusLabel(plugin) }}
              </span>
            </div>
            <p class="text-sm text-gray-400 mt-1">{{ plugin.description }}</p>
            <p class="text-xs text-gray-500 mt-2">
              {{ t('plugin.version') }}: {{ plugin.version }} |
              {{ t('plugin.author') }}: {{ plugin.author }}
            </p>
          </div>
          <button
            v-if="plugin.status === 'ready' || plugin.status === 'disabled'"
            @click="togglePlugin(plugin)"
            :disabled="toggling === plugin.id"
            class="ml-4 px-3 py-1 text-sm rounded transition"
            :class="{
              'bg-red-600 hover:bg-red-700 text-white': plugin.enabled,
              'bg-green-600 hover:bg-green-700 text-white': !plugin.enabled
            }"
          >
            {{ toggling === plugin.id ? t('plugin.toggling') : (plugin.enabled ? t('plugin.disable') : t('plugin.enable')) }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

interface Plugin {
  id: string
  name: string
  description: string
  version: string
  author: string
  enabled: boolean
  status: 'ready' | 'disabled' | 'error'
}

const { t } = useI18n()

const plugins = ref<Plugin[]>([])
const loading = ref(true)
const toggling = ref<string | null>(null)

async function loadPlugins() {
  loading.value = true
  try {
    const result = await invoke<Plugin[]>('list_discovered_plugins')
    plugins.value = Array.isArray(result) ? result : []
  } catch (error) {
    console.error('Failed to load plugins:', error)
    plugins.value = []
  } finally {
    loading.value = false
  }
}

async function togglePlugin(plugin: Plugin) {
  toggling.value = plugin.id
  try {
    if (plugin.enabled) {
      await invoke('disable_plugin', { pluginId: plugin.id })
    } else {
      await invoke('enable_plugin', { pluginId: plugin.id })
    }
    await loadPlugins()
  } catch (error) {
    console.error('Failed to toggle plugin:', error)
  } finally {
    toggling.value = null
  }
}

async function openPluginsDir() {
  try {
    await invoke('open_plugins_directory')
  } catch (error) {
    console.error('Failed to open plugins directory:', error)
  }
}

function getStatusLabel(plugin: Plugin) {
  if (plugin.status === 'error') {
    return t('plugin.statusError')
  }
  if (plugin.status === 'disabled' || !plugin.enabled) {
    return t('plugin.statusDisabled')
  }
  return t('plugin.statusEnabled')
}

onMounted(() => {
  loadPlugins()
})
</script>