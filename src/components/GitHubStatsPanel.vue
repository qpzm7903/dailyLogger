<template>
  <div
    class="bg-dark/60 backdrop-blur-md rounded-2xl p-5 border border-gray-700/50 shadow-xl transition-all duration-200 hover:shadow-2xl"
  >
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-2">
        <span class="text-2xl">🐙</span>
        <h2 class="font-medium text-white">GitHub 今日活动</h2>
      </div>
      <button
        @click="refresh"
        :disabled="loading"
        class="px-3 py-1.5 text-xs bg-gray-700/50 hover:bg-gray-600 disabled:opacity-50 rounded-lg text-gray-300 transition-colors flex items-center gap-1"
        title="刷新数据"
      >
        <span :class="loading ? 'animate-spin' : ''">🔄</span>
        {{ loading ? '刷新中...' : '刷新' }}
      </button>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-6 text-gray-400">
      <span class="animate-pulse">加载中...</span>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="text-center py-6">
      <p class="text-red-400 text-sm mb-2">{{ error }}</p>
      <button
        @click="refresh"
        class="text-xs text-primary hover:underline"
      >
        重试
      </button>
    </div>

    <!-- Not Configured State -->
    <div v-else-if="!configured" class="text-center py-6 text-gray-400">
      <p class="text-sm">GitHub 未配置</p>
      <button
        @click="$emit('openSettings')"
        class="mt-2 text-xs text-primary hover:underline"
      >
        前往设置配置 GitHub Token
      </button>
    </div>

    <!-- Stats Content -->
    <div v-else-if="stats" class="space-y-4">
      <!-- Summary Stats -->
      <div class="grid grid-cols-3 gap-3">
        <div class="bg-darker/80 rounded-xl p-3 text-center">
          <p class="text-2xl font-bold text-white">{{ stats.commit_count }}</p>
          <p class="text-xs text-gray-400">提交</p>
        </div>
        <div class="bg-darker/80 rounded-xl p-3 text-center">
          <p class="text-2xl font-bold text-white">{{ stats.pr_count }}</p>
          <p class="text-xs text-gray-400">PR</p>
        </div>
        <div class="bg-darker/80 rounded-xl p-3 text-center">
          <p class="text-2xl font-bold text-white">{{ stats.estimated_hours.toFixed(1) }}</p>
          <p class="text-xs text-gray-400">预估工时 (h)</p>
        </div>
      </div>

      <!-- Active Repositories -->
      <div v-if="stats.active_repos.length > 0">
        <p class="text-xs text-gray-400 mb-2">活跃仓库</p>
        <div class="flex flex-wrap gap-1.5">
          <span
            v-for="repo in stats.active_repos"
            :key="repo"
            class="px-2 py-0.5 bg-gray-700/50 rounded-full text-xs text-gray-300"
          >
            {{ repo }}
          </span>
        </div>
      </div>

      <!-- Commits by Hour -->
      <div v-if="Object.keys(stats.commits_by_hour).length > 0">
        <p class="text-xs text-gray-400 mb-2">提交时间分布</p>
        <div class="space-y-1">
          <div
            v-for="hour in sortedHours"
            :key="hour"
            class="flex items-center gap-2 text-xs"
          >
            <span class="text-gray-500 w-12">{{ hour }}:00</span>
            <div class="flex-1 bg-gray-700/30 rounded-full h-2 overflow-hidden">
              <div
                class="bg-primary h-full rounded-full transition-all duration-300"
                :style="{ width: `${getCommitBarWidth(hour)}%` }"
              ></div>
            </div>
            <span class="text-gray-400 w-8 text-right">
              {{ stats.commits_by_hour[hour].length }}
            </span>
          </div>
        </div>
      </div>

      <!-- Pull Requests -->
      <div v-if="stats.pull_requests.length > 0">
        <p class="text-xs text-gray-400 mb-2">Pull Requests</p>
        <div class="space-y-1">
          <p
            v-for="pr in stats.pull_requests"
            :key="pr"
            class="text-sm text-gray-300 truncate"
          >
            {{ pr }}
          </p>
        </div>
      </div>

      <!-- No Activity Today -->
      <div
        v-if="stats.commit_count === 0 && stats.pr_count === 0"
        class="text-center py-4 text-gray-500"
      >
        今日暂无 GitHub 活动
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { GitHubWorkStatsResponse, GitHubWorkStatsJson } from '../types/tauri'

const emit = defineEmits<{
  openSettings: []
}>()

const loading = ref(true)
const error = ref<string | null>(null)
const configured = ref(false)
const stats = ref<GitHubWorkStatsJson | null>(null)

const sortedHours = computed(() => {
  if (!stats.value) return []
  return Object.keys(stats.value.commits_by_hour)
    .map(Number)
    .sort((a, b) => a - b)
})

const getCommitBarWidth = (hour: number): number => {
  if (!stats.value) return 0
  const maxCommits = Math.max(
    ...Object.values(stats.value.commits_by_hour).map((arr: string[]) => arr.length)
  )
  if (maxCommits === 0) return 0
  return (stats.value.commits_by_hour[hour].length / maxCommits) * 100
}

const fetchStats = async () => {
  loading.value = true
  error.value = null

  try {
    const response = await invoke<GitHubWorkStatsResponse>('get_github_work_stats')
    configured.value = response.configured
    stats.value = response.stats
  } catch (e) {
    console.error('Failed to fetch GitHub stats:', e)
    error.value = typeof e === 'string' ? e : '获取 GitHub 统计失败'
  } finally {
    loading.value = false
  }
}

const refresh = () => {
  fetchStats()
}

onMounted(() => {
  fetchStats()
})
</script>