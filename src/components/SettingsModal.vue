<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[500px] max-h-[80vh] overflow-y-auto border border-gray-700">
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">{{ $t('settings.title') }}</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>

      <div class="p-6 space-y-6">
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.apiConfig') }}</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">Base URL</label>
              <div class="flex items-center gap-2">
                <input
                  v-model="settings.api_base_url"
                  type="text"
                  placeholder="https://api.openai.com/v1"
                  class="flex-1 bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
                <!-- Ollama status indicator -->
                <span v-if="isOllama" class="text-xs text-green-400 whitespace-nowrap">🦙 Ollama</span>
              </div>
              <span class="text-xs text-gray-500 mt-1 block">
                {{ $t('settings.baseUrlOllamaHint') }}
              </span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">
                {{ $t('settings.apiKey') }}
                <span v-if="isOllama" class="text-gray-500">{{ $t('settings.apiKeyOllamaHint') }}</span>
              </label>
              <div class="relative">
                <input
                  v-model="settings.api_key"
                  :type="showApiKey ? 'text' : 'password'"
                  placeholder="sk-..."
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 pr-16 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
                <button
                  @click="showApiKey = !showApiKey"
                  type="button"
                  class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 transition-colors text-xs px-2 py-1 rounded hover:bg-gray-700"
                  :title="showApiKey ? $t('common.hide') : $t('common.show')"
                >{{ showApiKey ? $t('common.hide') : $t('common.show') }}</button>
              </div>
            </div>
            <!-- Test Connection Button -->
            <div class="pt-2">
              <div class="flex items-center gap-2">
                <button
                  @click="testConnection"
                  :disabled="isTestingConnection || !settings.api_base_url || !settings.model_name || (!isOllama && !settings.api_key)"
                  class="px-3 py-1.5 text-sm bg-gray-700 hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
                >
                  {{ isTestingConnection ? $t('settings.testing') : $t('settings.testConnection') }}
                </button>
                <!-- Ollama model fetch button -->
                <button
                  v-if="isOllama"
                  @click="fetchOllamaModels"
                  :disabled="isLoadingOllamaModels || !settings.api_base_url"
                  class="px-3 py-1.5 text-sm bg-purple-700 hover:bg-purple-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
                >
                  {{ isLoadingOllamaModels ? $t('settings.fetching') : $t('settings.fetchModels') }}
                </button>
              </div>
              <span v-if="connectionTestResult" :class="connectionTestResult.success ? 'text-green-400' : 'text-red-400'" class="ml-2 text-xs">
                {{ connectionTestResult.message }}
                <span v-if="connectionTestResult.latency_ms">({{ connectionTestResult.latency_ms }}ms)</span>
              </span>
              <!-- Ollama model list -->
              <div v-if="isOllama" class="mt-3">
                <div class="flex items-center justify-between mb-1">
                  <label class="text-xs text-gray-300">{{ $t('settings.selectModel') }}</label>
                  <button
                    @click="fetchOllamaModels"
                    :disabled="isLoadingOllamaModels"
                    type="button"
                    class="text-xs text-primary hover:text-primary/80 disabled:opacity-50 transition-colors"
                  >
                    {{ isLoadingOllamaModels ? '...' : $t('settings.refreshModels') }}
                  </button>
                </div>

                <!-- Pull model input -->
                <div class="flex gap-2 mb-2">
                  <input
                    v-model="pullModelName"
                    type="text"
                    :placeholder="$t('settings.pullModelPlaceholder')"
                    class="flex-1 bg-darker border border-gray-700 rounded px-2 py-1 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                  />
                  <select
                    v-model="pullModelQuantization"
                    class="bg-darker border border-gray-700 rounded px-2 py-1 text-xs text-gray-100 focus:border-primary focus:outline-none"
                    :title="$t('settings.quantizationTooltip')"
                  >
                    <option value="">{{ $t('settings.defaultQuantization') }}</option>
                    <option value="q4_0">q4_0 ({{ $t('settings.smallest') }})</option>
                    <option value="q4_1">q4_1</option>
                    <option value="q5_0">q5_0</option>
                    <option value="q5_1">q5_1</option>
                    <option value="q8_0">q8_0 ({{ $t('settings.largest') }})</option>
                    <option value="f16">f16 ({{ $t('settings.noCompression') }})</option>
                  </select>
                  <button
                    @click="pullModel"
                    :disabled="isPullingModel || !pullModelName"
                    type="button"
                    class="px-2 py-1 text-xs rounded bg-primary hover:bg-primary/80 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                  >
                    {{ isPullingModel ? $t('settings.pulling') : $t('settings.pullModel') }}
                  </button>
                </div>

                <!-- Model list -->
                <div v-if="ollamaModels.length > 0" class="flex flex-wrap gap-2">
                  <div
                    v-for="model in ollamaModels"
                    :key="model.name"
                    class="flex items-center gap-1 px-2 py-1 text-xs rounded border transition-colors"
                    :class="settings.model_name === model.name ? 'bg-primary border-primary text-white' : 'bg-darker border-gray-600 text-gray-300 hover:border-primary'"
                  >
                    <button
                      @click="selectOllamaModel(model.name)"
                      type="button"
                      class="hover:text-white transition-colors"
                    >
                      {{ model.name }}<span v-if="model.size" class="text-gray-400 ml-1">({{ model.size }})</span>
                    </button>
                    <button
                      @click="openCopyModelModal(model.name)"
                      type="button"
                      class="ml-1 text-gray-400 hover:text-blue-400 transition-colors"
                      :title="$t('settings.copyModel')"
                    >⧉</button>
                    <button
                      @click="deleteModel(model.name)"
                      :disabled="isDeletingModel === model.name"
                      type="button"
                      class="ml-1 text-gray-400 hover:text-red-400 disabled:opacity-50 transition-colors"
                      :title="$t('settings.deleteModel')"
                    >×</button>
                  </div>
                </div>
                <p v-else-if="!isLoadingOllamaModels" class="text-xs text-gray-500">{{ $t('settings.noModelsFound') }}</p>

                <!-- Running models status -->
                <div v-if="isOllama" class="mt-3 pt-3 border-t border-gray-700">
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-xs text-gray-400">{{ $t('settings.runningModels') }}</span>
                    <button
                      @click="fetchRunningModels"
                      :disabled="isLoadingRunningModels"
                      type="button"
                      class="text-xs text-primary hover:text-primary/80 disabled:opacity-50 transition-colors"
                    >
                      {{ isLoadingRunningModels ? '...' : $t('settings.refreshModels') }}
                    </button>
                  </div>
                  <div v-if="runningModels.length > 0" class="space-y-1">
                    <div
                      v-for="model in runningModels"
                      :key="model.name"
                      class="flex items-center justify-between px-2 py-1 text-xs bg-green-900/30 border border-green-800 rounded"
                    >
                      <span class="text-green-300">{{ model.name }}</span>
                      <span v-if="model.size_vram" class="text-green-400 text-xs">
                        {{ $t('settings.vramUsage', { size: formatModelSize(model.size_vram) }) }}
                      </span>
                    </div>
                  </div>
                  <p v-else-if="!isLoadingRunningModels" class="text-xs text-gray-500">{{ $t('settings.noRunningModels') }}</p>
                </div>

                <!-- Create custom model button -->
                <div v-if="isOllama && ollamaModels.length > 0" class="mt-3 pt-3 border-t border-gray-700">
                  <button
                    @click="showCreateModelModal = true"
                    type="button"
                    class="w-full px-3 py-2 text-xs bg-gradient-to-r from-purple-700 to-indigo-700 hover:from-purple-600 hover:to-indigo-600 rounded-lg transition-colors"
                  >
                    {{ $t('settings.createCustomModel') }}
                  </button>
                </div>

                <!-- Fine-tuning button -->
                <div v-if="isOllama" class="mt-2">
                  <button
                    @click="showFineTuningModal = true"
                    type="button"
                    class="w-full px-3 py-2 text-xs bg-gradient-to-r from-emerald-700 to-teal-700 hover:from-emerald-600 hover:to-teal-600 rounded-lg transition-colors"
                  >
                    {{ $t('settings.fineTuning') }}
                  </button>
                </div>
              </div>

              <!-- Running models section -->
              <div v-if="isOllama" class="mt-3 pt-3 border-t border-gray-700">
                <div class="flex items-center justify-between mb-1">
                  <label class="text-xs text-gray-300">{{ $t('settings.runningModels') }}</label>
                  <button
                    @click="fetchRunningModels"
                    :disabled="isLoadingRunningModels"
                    type="button"
                    class="text-xs text-primary hover:text-primary/80 disabled:opacity-50 transition-colors"
                  >
                    {{ isLoadingRunningModels ? $t('settings.loadingRunningModels') : $t('settings.refreshRunning') }}
                  </button>
                </div>
                <div v-if="runningModels.length > 0" class="flex flex-wrap gap-2">
                  <div
                    v-for="model in runningModels"
                    :key="model.name"
                    class="flex items-center gap-1 px-2 py-1 text-xs rounded bg-green-900/30 border border-green-700 text-green-300"
                  >
                    <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
                    <span>{{ model.name }}</span>
                    <span v-if="model.size_vram" class="text-gray-400 ml-1">({{ formatModelSize(model.size_vram) }} VRAM)</span>
                  </div>
                </div>
                <p v-else-if="!isLoadingRunningModels" class="text-xs text-gray-500">{{ $t('settings.noRunningModels') }}</p>
              </div>

              <p v-if="ollamaModelError" class="text-xs text-red-400 mt-1">{{ ollamaModelError }}</p>
            </div>
          </div>
        </div>

        <div v-if="isDesktop">
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.screenshotAnalysis') }}</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.analysisModel') }}</label>
              <div class="flex items-center gap-2">
                <input
                  v-model="settings.model_name"
                  type="text"
                  placeholder="gpt-4o"
                  class="flex-1 bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
                <button
                  @click="getModelInfo('analysis')"
                  :disabled="isLoadingModelInfo || !settings.model_name"
                  type="button"
                  class="text-gray-400 hover:text-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors px-2"
                  :title="$t('settings.contextWindow', { size: '' })"
                >ℹ️</button>
              </div>
              <span v-if="analysisModelInfo?.context_window" class="text-xs text-gray-500 mt-1 block">
                {{ $t('settings.contextWindow', { size: analysisModelInfo.context_window / 1000 }) }}
              </span>
              <span v-else class="text-xs text-gray-500 mt-1 block">{{ $t('settings.visionRequired') }}</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.analysisPrompt') }}</label>
              <textarea
                v-model="settings.analysis_prompt"
                rows="4"
                :placeholder="$t('settings.analysisPromptPlaceholder')"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y"
              />
              <div class="flex gap-3 mt-2">
                <button
                  type="button"
                  @click="showDefaultPrompt"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  {{ $t('common.viewDefault') }}
                </button>
                <button
                  type="button"
                  @click="resetPrompt"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  {{ $t('common.resetDefault') }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.dailyReport') }}</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.reportTitleFormat') }}</label>
              <input
                v-model="settings.summary_title_format"
                type="text"
                :placeholder="$t('settings.reportTitlePlaceholder')"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-500 mt-1 block">{{ $t('settings.reportTitleHint') }}</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.reportModel') }}</label>
              <div class="flex items-center gap-2">
                <input
                  v-model="settings.summary_model_name"
                  type="text"
                  :placeholder="$t('settings.reportModelPlaceholder')"
                  class="flex-1 bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
                <button
                  @click="getModelInfo('summary')"
                  :disabled="isLoadingModelInfo || !settings.summary_model_name"
                  type="button"
                  class="text-gray-400 hover:text-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors px-2"
                  :title="$t('settings.contextWindow', { size: '' })"
                >ℹ️</button>
              </div>
              <span v-if="summaryModelInfo?.context_window" class="text-xs text-gray-500 mt-1 block">
                {{ $t('settings.contextWindow', { size: summaryModelInfo.context_window / 1000 }) }}
              </span>
              <span v-else class="text-xs text-gray-500 mt-1 block">{{ $t('settings.textModelHint') }}</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.reportPrompt') }}</label>
              <textarea
                v-model="settings.summary_prompt"
                rows="4"
                :placeholder="$t('settings.reportPromptPlaceholder')"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y"
              />
              <div class="flex gap-3 mt-2">
                <button
                  type="button"
                  @click="showDefaultSummaryPrompt"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  {{ $t('common.viewDefault') }}
                </button>
                <button
                  type="button"
                  @click="resetSummaryPrompt"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  {{ $t('common.resetDefault') }}
                </button>
                <button
                  type="button"
                  @click="showTemplateLibrary"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  {{ $t('common.templateLibrary') }}
                </button>
                <button
                  type="button"
                  @click="exportTemplate"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  {{ $t('common.exportTemplate') }}
                </button>
                <button
                  type="button"
                  @click="importTemplate"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  {{ $t('common.importTemplate') }}
                </button>
              </div>
            </div>
            <div class="flex items-center gap-2 pt-1">
              <input
                v-model="settings.include_manual_records"
                type="checkbox"
                id="include_manual_records"
                class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
              />
              <label for="include_manual_records" class="text-xs text-gray-300 cursor-pointer">
                {{ $t('settings.includeQuickNotes') }}
              </label>
              <span class="text-xs text-gray-500">{{ $t('settings.includeQuickNotesHint') }}</span>
            </div>
          </div>
        </div>

        <!-- AI-004: 标签分类配置 -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.tagCategories') }}</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.customTagCategories') }}</label>
              <textarea
                v-model="tagCategoriesText"
                rows="4"
                :placeholder="$t('settings.tagCategoriesPlaceholder')"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y font-mono"
              />
              <span class="text-xs text-gray-500 mt-1 block">{{ $t('settings.tagCategoriesHint') }}</span>
              <div class="flex gap-3 mt-2">
                <button
                  type="button"
                  @click="showDefaultTagCategories"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  {{ $t('settings.viewDefaultTags') }}
                </button>
                <button
                  type="button"
                  @click="resetTagCategories"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  {{ $t('common.resetDefault') }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.timeStrategy') }}</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.screenshotInterval') }}</label>
              <input
                v-model.number="settings.screenshot_interval"
                type="number"
                min="1"
                max="60"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.summaryTime') }}</label>
              <input
                v-model="settings.summary_time"
                type="time"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.smartDedup') }}</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.changeThreshold') }}</label>
              <input
                v-model.number="settings.change_threshold"
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
                v-model.number="settings.max_silent_minutes"
                type="number"
                min="5"
                max="120"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-500 mt-1 block">{{ $t('settings.maxSilentTimeHint') }}</span>
            </div>
          </div>
        </div>

        <!-- SMART-002: 智能静默阈值调整 -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.silentThresholdAdjust') }}</h3>
          <div class="space-y-3">
            <div class="flex items-center gap-2">
              <input
                v-model="settings.auto_adjust_silent"
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
            <div v-if="!settings.auto_adjust_silent" class="bg-darker rounded-lg p-3 border border-gray-700">
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-gray-400">{{ $t('settings.manualModeEnabled') }}</span>
                <span class="text-xs text-primary">{{ settings.max_silent_minutes }} {{ $t('settings.timeStrategy').replace('(分钟)', '').trim() }}</span>
              </div>
              <span class="text-xs text-gray-500">
                {{ $t('settings.autoAdjustHint') }}
              </span>
            </div>
            <div v-else class="bg-darker rounded-lg p-3 border border-gray-700">
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-gray-400">{{ $t('settings.learningStatus') }}</span>
                <span class="text-xs text-green-400">{{ $t('settings.autoLearning') }}</span>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-xs text-gray-400">{{ $t('settings.currentThreshold') }}</span>
                <span class="text-xs text-primary">{{ settings.max_silent_minutes }} {{ $t('settings.timeStrategy').replace('(分钟)', '').trim() }}</span>
              </div>
              <span class="text-xs text-gray-500 mt-2 block">
                {{ $t('settings.autoAdjustRangeHint') }}
              </span>
            </div>
          </div>
        </div>

        <!-- SMART-003: 工作时间自动识别 -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.workTimeDetection') }}</h3>
          <div class="space-y-3">
            <div class="flex items-center gap-2">
              <input
                v-model="settings.auto_detect_work_time"
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
            <div class="flex items-center gap-2 pt-2" v-if="settings.auto_detect_work_time">
              <input
                v-model="settings.use_custom_work_time"
                type="checkbox"
                id="use_custom_work_time"
                class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
              />
              <label for="use_custom_work_time" class="text-xs text-gray-300 cursor-pointer">
                {{ $t('settings.useCustomWorkTime') }}
              </label>
            </div>

            <!-- Custom work time inputs -->
            <div v-if="settings.auto_detect_work_time && settings.use_custom_work_time" class="grid grid-cols-2 gap-3 pt-2">
              <div>
                <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.startTime') }}</label>
                <input
                  v-model="settings.custom_work_time_start"
                  type="time"
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
              </div>
              <div>
                <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.endTime') }}</label>
                <input
                  v-model="settings.custom_work_time_end"
                  type="time"
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
              </div>
            </div>

            <!-- Work time status display -->
            <div v-if="settings.auto_detect_work_time && !settings.use_custom_work_time" class="bg-darker rounded-lg p-3 border border-gray-700">
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-gray-400">{{ $t('settings.learningStatus') }}</span>
                <span v-if="workTimeStatus" class="text-xs" :class="workTimeStatus.is_work_time ? 'text-green-400' : 'text-yellow-400'">
                  {{ workTimeStatus.is_work_time ? $t('settings.working') : $t('settings.nonWorkTime') }}
                </span>
              </div>
              <div v-if="workTimeStatus" class="space-y-1">
                <div class="flex items-center justify-between">
                  <span class="text-xs text-gray-400">{{ $t('settings.daysLearned') }}</span>
                  <span class="text-xs text-primary">{{ workTimeStatus.learning_progress.days_learned }} / {{ workTimeStatus.learning_progress.min_days_required }} 天</span>
                </div>
                <div v-if="workTimeStatus.learning_progress.days_learned >= workTimeStatus.learning_progress.min_days_required" class="flex items-center justify-between">
                  <span class="text-xs text-gray-400">{{ $t('settings.detectedWorkTime') }}</span>
                  <span class="text-xs text-gray-300">{{ formatWorkTimePeriods(workTimeStatus.detected_periods) }}</span>
                </div>
                <div v-else class="mt-2">
                  <div class="w-full bg-gray-700 rounded-full h-1.5">
                    <div
                      class="bg-primary h-1.5 rounded-full transition-all"
                      :style="{ width: `${Math.min(100, (workTimeStatus.learning_progress.days_learned / workTimeStatus.learning_progress.min_days_required) * 100)}%` }"
                    ></div>
                  </div>
                  <span class="text-xs text-gray-500 mt-1 block">{{ $t('settings.continueLearningHint') }}</span>
                </div>
              </div>
            </div>

            <!-- Disabled info -->
            <div v-if="!settings.auto_detect_work_time" class="bg-darker rounded-lg p-3 border border-gray-700">
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-gray-400">{{ $t('settings.workTimeDetectionOff') }}</span>
                <span class="text-xs text-gray-500">{{ $t('settings.off') }}</span>
              </div>
              <span class="text-xs text-gray-500">
                {{ $t('settings.workTimeOffHint') }}
              </span>
            </div>
          </div>
        </div>

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
              <span class="text-xs text-gray-500 mt-1 block">{{ $t('settings.filterMatchHint') }}</span>
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
              <span class="text-xs text-gray-500 mt-1 block">{{ $t('settings.filterMatchHint') }}</span>
            </div>
            <div class="flex items-center gap-2 pt-1">
              <input
                v-model="settings.use_whitelist_only"
                type="checkbox"
                id="use_whitelist_only"
                class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
              />
              <label for="use_whitelist_only" class="text-xs text-gray-300 cursor-pointer">
                {{ $t('settings.whitelistOnly') }}
              </label>
              <span class="text-xs text-gray-500">{{ $t('settings.whitelistOnlyHint') }}</span>
            </div>
          </div>
        </div>

        <!-- SMART-004: 显示器设置 -->
        <div v-if="isScreenshotEnabled && isDesktop">
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.displaySettings') }}</h3>
          <div class="space-y-3">
            <!-- 多显示器时显示捕获模式选择 -->
            <div v-if="monitors?.length > 1" class="space-y-2">
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.captureMode') }}</label>
              <div class="flex flex-wrap gap-4">
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    v-model="settings.capture_mode"
                    value="primary"
                    class="w-4 h-4 border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0"
                  />
                  <span class="text-sm text-gray-300">{{ $t('settings.primaryMonitor') }}</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    v-model="settings.capture_mode"
                    value="secondary"
                    class="w-4 h-4 border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0"
                  />
                  <span class="text-sm text-gray-300">{{ $t('settings.secondaryMonitor') }}</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    v-model="settings.capture_mode"
                    value="all"
                    class="w-4 h-4 border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0"
                  />
                  <span class="text-sm text-gray-300">{{ $t('settings.allMonitors') }}</span>
                </label>
              </div>
            </div>

            <!-- 显示器列表 -->
            <div v-if="monitors?.length > 1" class="space-y-1">
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.connectedDisplays') }}</label>
              <div
                v-for="m in monitors"
                :key="m.index"
                class="flex items-center gap-2 text-sm bg-darker rounded-lg px-3 py-2 border border-gray-700"
              >
                <span class="text-gray-300">{{ m.name }}</span>
                <span class="text-gray-500">{{ m.resolution }}</span>
                <span v-if="m.is_primary" class="text-xs bg-primary/20 text-primary px-1.5 py-0.5 rounded">{{ $t('settings.primary') }}</span>
                <!-- 副显示器模式下可选择 -->
                <button
                  v-if="settings.capture_mode === 'secondary' && !m.is_primary && m.index !== undefined"
                  type="button"
                  @click="settings.selected_monitor_index = m.index"
                  :class="[
                    'ml-auto text-xs px-2 py-1 rounded transition-colors',
                    settings.selected_monitor_index === m.index
                      ? 'bg-primary text-white'
                      : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                  ]"
                >
                  {{ settings.selected_monitor_index === m.index ? $t('settings.selected') : $t('settings.select') }}
                </button>
              </div>
            </div>

            <!-- 单显示器提示 -->
            <div v-if="monitors?.length === 1" class="bg-darker rounded-lg p-3 border border-gray-700">
              <div class="flex items-center justify-between mb-1">
                <span class="text-xs text-gray-400">{{ $t('settings.currentDisplay') }}</span>
                <span class="text-xs text-gray-300">{{ monitors[0]?.name || $t('settings.unknown') }}</span>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-xs text-gray-400">{{ $t('settings.resolution') }}</span>
                <span class="text-xs text-gray-300">{{ monitors[0]?.resolution || $t('settings.unknown') }}</span>
              </div>
            </div>

            <!-- 加载中或错误状态 -->
            <div v-if="isLoadingMonitors" class="text-xs text-gray-500">
              {{ $t('settings.loadingDisplays') }}
            </div>
            <div v-if="monitorError" class="text-xs text-red-400">
              {{ monitorError }}
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.outputConfig') }}</h3>
          <div class="space-y-3">
            <label class="text-xs text-gray-300 block">{{ $t('settings.obsidianVaults') }}</label>
            <!-- Vault list -->
            <div v-for="(vault, index) in vaults" :key="index"
              class="flex items-center gap-2 bg-darker border border-gray-700 rounded-lg px-3 py-2">
              <button @click="setDefaultVault(index)" class="text-xs shrink-0"
                :class="vault.is_default ? 'text-primary font-bold' : 'text-gray-500 hover:text-gray-300'">
                {{ vault.is_default ? '★' : '☆' }}
              </button>
              <div class="flex-1 min-w-0">
                <div class="text-sm text-gray-100 truncate">{{ vault.name }}</div>
                <div class="text-xs text-gray-500 truncate">{{ vault.path }}</div>
              </div>
              <button @click="removeVault(index)" class="text-gray-500 hover:text-red-400 text-xs shrink-0">✕</button>
            </div>
            <div v-if="vaults.length === 0" class="text-xs text-gray-500 py-2">
              {{ $t('settings.noVaultConfigured') }}
            </div>
            <!-- Add vault form -->
            <div class="flex gap-2">
              <input v-model="newVaultName" type="text" :placeholder="$t('common.name')"
                class="w-1/3 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
              <input v-model="newVaultPath" type="text" :placeholder="$t('common.path')"
                class="flex-1 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
              <button @click="addVault" :disabled="!newVaultName.trim() || !newVaultPath.trim()"
                class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-30 rounded-lg text-xs text-primary transition-colors shrink-0">
                {{ $t('common.add') }}
              </button>
            </div>
          </div>
        </div>

        <div>
          <label class="text-xs text-gray-300 block mb-2">{{ $t('settings.logseqGraphs') }}</label>
          <!-- Graph list -->
          <div v-for="(graph, index) in graphs" :key="index"
            class="flex items-center gap-2 bg-darker border border-gray-700 rounded-lg px-3 py-2 mb-2">
            <button @click="setDefaultGraph(index)" class="text-xs shrink-0"
              :class="graph.is_default ? 'text-primary font-bold' : 'text-gray-500 hover:text-gray-300'">
              {{ graph.is_default ? '★' : '☆' }}
            </button>
            <div class="flex-1 min-w-0">
              <div class="text-sm text-gray-100 truncate">{{ graph.name }}</div>
              <div class="text-xs text-gray-500 truncate">{{ graph.path }}</div>
            </div>
            <button @click="removeGraph(index)" class="text-gray-500 hover:text-red-400 text-xs shrink-0">✕</button>
          </div>
          <div v-if="graphs.length === 0" class="text-xs text-gray-500 py-2 mb-2">
            {{ $t('settings.noGraphConfigured') }}
          </div>
          <!-- Add graph form -->
          <div class="flex gap-2">
            <input v-model="newGraphName" type="text" :placeholder="$t('common.name')"
              class="w-1/3 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
            <input v-model="newGraphPath" type="text" :placeholder="$t('common.path')"
              class="flex-1 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
            <button @click="addGraph" :disabled="!newGraphName.trim() || !newGraphPath.trim()"
              class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-30 rounded-lg text-xs text-primary transition-colors shrink-0">
              {{ $t('common.add') }}
            </button>
          </div>
        </div>

        <!-- INT-001: Notion Integration -->
        <div>
          <label class="text-xs text-gray-300 block mb-2">{{ $t('settings.notionIntegration') }}</label>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.notionApiKey') }}</label>
              <input v-model="settings.notion_api_key" type="password" :placeholder="$t('settings.notionApiKeyPlaceholder')"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.notionDatabaseId') }}</label>
              <input v-model="settings.notion_database_id" type="text" :placeholder="$t('settings.notionDatabaseIdPlaceholder')"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
            </div>
            <div class="flex gap-2">
              <button @click="testNotionConnection" :disabled="isTestingNotionConnection"
                class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-50 rounded-lg text-xs text-primary transition-colors">
                {{ isTestingNotionConnection ? $t('common.testing') : $t('common.testConnection') }}
              </button>
              <span v-if="notionConnectionStatus" class="text-xs"
                :class="notionConnectionStatus === 'success' ? 'text-green-400' : 'text-red-400'">
                {{ notionConnectionStatus === 'success' ? '✓ ' + $t('common.connected') : '✗ ' + $t('common.failed') }}
              </span>
            </div>
            <p class="text-xs text-gray-500">
              {{ $t('settings.notionHint') }}
            </p>
          </div>
        </div>

        <!-- INT-003: GitHub Work Time Statistics -->
        <div>
          <label class="text-xs text-gray-300 block mb-2">{{ $t('settings.githubWorkTime') }}</label>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.githubToken') }}</label>
              <input v-model="settings.github_token" type="password" :placeholder="$t('settings.githubTokenPlaceholder')"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.githubRepos') }}</label>
              <textarea
                :value="githubReposText"
                @input="updateGithubRepos"
                rows="3"
                placeholder="owner/repo1&#10;owner/repo2"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-none font-mono"></textarea>
            </div>
            <div class="flex gap-2">
              <button @click="testGithubConnection" :disabled="isTestingGithubConnection"
                class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-50 rounded-lg text-xs text-primary transition-colors">
                {{ isTestingGithubConnection ? $t('common.testing') : $t('common.testConnection') }}
              </button>
              <span v-if="githubConnectionStatus" class="text-xs"
                :class="githubConnectionStatus === 'success' ? 'text-green-400' : 'text-red-400'">
                {{ githubConnectionStatus === 'success' ? '✓ ' + $t('common.connected') : '✗ ' + $t('common.failed') }}
              </span>
            </div>
            <p class="text-xs text-gray-500">
              {{ $t('settings.githubHint') }}
            </p>
          </div>
        </div>

        <!-- INT-004: Slack Notification -->
        <div>
          <label class="text-xs text-gray-300 block mb-2">{{ $t('settings.slackNotification') }}</label>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.slackWebhookUrl') }}</label>
              <input v-model="settings.slack_webhook_url" type="password" :placeholder="$t('settings.slackWebhookPlaceholder')"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
            </div>
            <div class="flex gap-2">
              <button @click="testSlackConnection" :disabled="isTestingSlackConnection"
                class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-50 rounded-lg text-xs text-primary transition-colors">
                {{ isTestingSlackConnection ? $t('common.testing') : $t('common.testConnection') }}
              </button>
              <span v-if="slackConnectionStatus" class="text-xs"
                :class="slackConnectionStatus === 'success' ? 'text-green-400' : 'text-red-400'">
                {{ slackConnectionStatus === 'success' ? '✓ ' + $t('common.connected') : '✗ ' + $t('common.failed') }}
              </span>
            </div>
            <p class="text-xs text-gray-500">
              {{ $t('settings.slackHint') }}
            </p>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.language') }}</h3>
          <div class="space-y-3">
            <div class="flex gap-2">
              <button
                @click="changeLanguage('en')"
                type="button"
                class="flex-1 px-3 py-2 text-sm rounded-lg border transition-colors"
                :class="locale === 'en' ? 'bg-primary border-primary text-white' : 'bg-darker border-gray-600 text-gray-300 hover:border-primary'"
              >
                {{ $t('settings.languageEn') }}
              </button>
              <button
                @click="changeLanguage('zh-CN')"
                type="button"
                class="flex-1 px-3 py-2 text-sm rounded-lg border transition-colors"
                :class="locale === 'zh-CN' ? 'bg-primary border-primary text-white' : 'bg-darker border-gray-600 text-gray-300 hover:border-primary'"
              >
                {{ $t('settings.languageZhCN') }}
              </button>
            </div>
            <p class="text-xs text-gray-500">{{ $t('settings.languageHint') }}</p>
          </div>
        </div>

        <div v-if="isDesktop">
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.shortcuts') }}</h3>
          <div class="bg-darker rounded-lg px-3 py-2 text-sm text-gray-400 border border-gray-700">
            {{ $t('settings.quickNoteShortcut') }}
          </div>
        </div>

        <!-- Plugin Management -->
        <div>
          <PluginPanel />
        </div>

        <!-- Team Collaboration -->
        <div>
          <TeamPanel />
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">{{ $t('settings.debugTools') }}</h3>
          <div class="space-y-3">
            <button
              @click="exportLogs"
              :disabled="isExportingLogs"
              class="w-full px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:opacity-50 rounded-lg text-sm text-gray-200 transition-colors flex items-center justify-center gap-2"
            >
              {{ isExportingLogs ? $t('settings.exporting') : '📤 ' + $t('settings.exportLogs') }}
            </button>
            <span v-if="exportError" class="text-xs text-red-400 block">{{ exportError }}</span>
          </div>
        </div>
      </div>

      <!-- Default Prompt Modal -->
      <div v-if="showDefaultPromptModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showDefaultPromptModal = false">
        <div class="bg-dark rounded-2xl w-[500px] max-h-[80vh] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">{{ t('settings.defaultAnalysisPrompt') }}</h3>
            <button @click="showDefaultPromptModal = false" class="text-gray-400 hover:text-white">✕</button>
          </div>
          <div class="p-6 overflow-y-auto max-h-[60vh]">
            <pre class="text-sm text-gray-300 whitespace-pre-wrap bg-darker p-4 rounded-lg">{{ defaultPromptContent }}</pre>
          </div>
          <div class="px-6 py-4 border-t border-gray-700 flex justify-end">
            <button
              @click="showDefaultPromptModal = false"
              class="px-4 py-2 bg-primary rounded-lg text-sm font-medium text-white hover:bg-blue-600 transition-colors"
            >
              {{ t('settings.close') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Default Summary Prompt Modal -->
      <div v-if="showDefaultSummaryPromptModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showDefaultSummaryPromptModal = false">
        <div class="bg-dark rounded-2xl w-[500px] max-h-[80vh] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">{{ t('settings.defaultReportPrompt') }}</h3>
            <button @click="showDefaultSummaryPromptModal = false" class="text-gray-400 hover:text-white">✕</button>
          </div>
          <div class="p-6 overflow-y-auto max-h-[60vh]">
            <pre class="text-sm text-gray-300 whitespace-pre-wrap bg-darker p-4 rounded-lg">{{ defaultSummaryPromptContent }}</pre>
          </div>
          <div class="px-6 py-4 border-t border-gray-700 flex justify-end">
            <button
              @click="showDefaultSummaryPromptModal = false"
              class="px-4 py-2 bg-primary rounded-lg text-sm font-medium text-white hover:bg-blue-600 transition-colors"
            >
              {{ t('settings.close') }}
            </button>
          </div>
        </div>
      </div>

      <!-- AI-004: Default Tag Categories Modal -->
      <div v-if="showDefaultTagCategoriesModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showDefaultTagCategoriesModal = false">
        <div class="bg-dark rounded-2xl w-[400px] max-h-[80vh] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">{{ t('settings.defaultTagCategories') }}</h3>
            <button @click="showDefaultTagCategoriesModal = false" class="text-gray-400 hover:text-white">✕</button>
          </div>
          <div class="p-6 overflow-y-auto max-h-[60vh]">
            <div class="flex flex-wrap gap-2">
              <span
                v-for="tag in defaultTagCategoriesContent"
                :key="tag"
                class="px-3 py-1 bg-primary/20 text-primary rounded-full text-sm"
              >
                {{ tag }}
              </span>
            </div>
          </div>
          <div class="px-6 py-4 border-t border-gray-700 flex justify-end">
            <button
              @click="showDefaultTagCategoriesModal = false"
              class="px-4 py-2 bg-primary rounded-lg text-sm font-medium text-white hover:bg-blue-600 transition-colors"
            >
              {{ t('settings.close') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Template Library Modal -->
      <div v-if="showTemplateLibraryModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showTemplateLibraryModal = false">
        <div class="bg-dark rounded-2xl w-[500px] max-h-[80vh] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">{{ t('common.templateLibrary') }}</h3>
            <button @click="showTemplateLibraryModal = false" class="text-gray-400 hover:text-white">✕</button>
          </div>
          <div class="p-6 overflow-y-auto max-h-[60vh] space-y-4">
            <div
              v-for="template in presetTemplates"
              :key="template.id"
              class="bg-darker rounded-lg p-4 border border-gray-700 hover:border-primary transition-colors cursor-pointer"
              @click="applyTemplate(template)"
            >
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="text-sm font-medium text-gray-200">{{ template.name }}</h4>
                  <p class="text-xs text-gray-400 mt-1">{{ template.description }}</p>
                </div>
                <button
                  class="px-3 py-1 bg-primary/20 text-primary text-xs rounded hover:bg-primary hover:text-white transition-colors"
                  @click.stop="applyTemplate(template)"
                >
                  应用
                </button>
              </div>
            </div>
          </div>
          <div class="px-6 py-4 border-t border-gray-700 flex justify-end">
            <button
              @click="showTemplateLibraryModal = false"
              class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg text-sm text-gray-200 transition-colors"
            >
              {{ t('settings.close') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Create Custom Ollama Model Modal -->
      <div v-if="showCreateModelModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showCreateModelModal = false">
        <div class="bg-dark rounded-2xl w-[450px] max-h-[80vh] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">{{ $t('settings.createModelTitle') }}</h3>
            <button @click="showCreateModelModal = false" class="text-gray-400 hover:text-white">✕</button>
          </div>
          <div class="p-6 space-y-4">
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.createModelName') }}</label>
              <input
                v-model="createModelParams.name"
                type="text"
                :placeholder="$t('settings.createModelNamePlaceholder')"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.createModelBase') }}</label>
              <select
                v-model="createModelParams.from"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
              >
                <option value="" disabled>{{ $t('settings.createModelBasePlaceholder') }}</option>
                <option v-for="model in ollamaModels" :key="model.name" :value="model.name">{{ model.name }}</option>
              </select>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.createModelSystem') }}</label>
              <textarea
                v-model="createModelParams.system"
                rows="3"
                :placeholder="$t('settings.createModelSystemPlaceholder')"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y"
              />
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.createModelTemperature') }}</label>
                <input
                  v-model.number="createModelParams.temperature"
                  type="number"
                  min="0"
                  max="2"
                  step="0.1"
                  placeholder="0.7"
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
              </div>
              <div>
                <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.createModelContext') }}</label>
                <input
                  v-model.number="createModelParams.num_ctx"
                  type="number"
                  min="512"
                  step="512"
                  placeholder="4096"
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
              </div>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.createModelQuantize') }}</label>
              <select
                v-model="createModelParams.quantize"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
              >
                <option value="">{{ $t('settings.createModelQuantizeNone') }}</option>
                <option value="q4_0">q4_0 (4-bit, fastest)</option>
                <option value="q4_1">q4_1 (4-bit, higher accuracy)</option>
                <option value="q5_0">q5_0 (5-bit, balanced)</option>
                <option value="q5_1">q5_1 (5-bit, higher accuracy)</option>
                <option value="q8_0">q8_0 (8-bit, highest accuracy)</option>
                <option value="q4_K_M">q4_K_M (4-bit K-quant, recommended)</option>
                <option value="q5_K_M">q5_K_M (5-bit K-quant)</option>
                <option value="q6_K">q6_K (6-bit K-quant)</option>
              </select>
              <p class="text-xs text-gray-500 mt-1">{{ $t('settings.createModelQuantizeHint') }}</p>
            </div>
          </div>
          <div class="px-6 py-4 border-t border-gray-700 flex justify-end gap-3">
            <button
              @click="showCreateModelModal = false"
              class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg text-sm text-gray-200 transition-colors"
            >
              {{ t('settings.cancel') }}
            </button>
            <button
              @click="createCustomModel"
              :disabled="isCreatingModel || !createModelParams.name || !createModelParams.from"
              class="px-4 py-2 bg-primary hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-sm font-medium text-white transition-colors"
            >
              {{ isCreatingModel ? $t('settings.creating') : $t('settings.createCustomModel') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Copy Model Modal -->
      <div v-if="showCopyModelModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showCopyModelModal = false">
        <div class="bg-dark rounded-2xl w-[400px] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">{{ $t('settings.copyModelTitle') }}</h3>
            <button @click="showCopyModelModal = false" class="text-gray-400 hover:text-white">✕</button>
          </div>
          <div class="p-6 space-y-4">
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.copyModelSource') }}</label>
              <input
                :value="copyModelSource"
                type="text"
                disabled
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-400 cursor-not-allowed"
              />
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.copyModelDestination') }}</label>
              <input
                v-model="copyModelDestination"
                type="text"
                :placeholder="$t('settings.copyModelDestinationPlaceholder')"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
            <p class="text-xs text-gray-500">{{ $t('settings.copyModelHint') }}</p>
          </div>
          <div class="px-6 py-4 border-t border-gray-700 flex justify-end gap-3">
            <button
              @click="showCopyModelModal = false"
              class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg text-sm text-gray-200 transition-colors"
            >
              {{ t('settings.cancel') }}
            </button>
            <button
              @click="copyModel"
              :disabled="isCopyingModel || !copyModelDestination.trim()"
              class="px-4 py-2 bg-primary hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-sm font-medium text-white transition-colors"
            >
              {{ isCopyingModel ? $t('settings.copying') : $t('settings.copyModelButton') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Fine-tuning Modal -->
      <div v-if="showFineTuningModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showFineTuningModal = false">
        <div class="bg-dark rounded-2xl w-[500px] max-h-[85vh] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">{{ $t('settings.fineTuningTitle') }}</h3>
            <button @click="showFineTuningModal = false" class="text-gray-400 hover:text-white">✕</button>
          </div>
          <div class="p-6 space-y-4 overflow-y-auto max-h-[60vh]">
            <!-- Training Data Section -->
            <div class="space-y-3">
              <h4 class="text-sm font-medium text-gray-300">{{ $t('settings.fineTuningTrainingData') }}</h4>
              <div class="grid grid-cols-2 gap-3">
                <label class="flex items-center gap-2 text-xs text-gray-300">
                  <input type="checkbox" v-model="fineTuningParams.includeAutoRecords" class="rounded bg-darker border-gray-600" />
                  {{ $t('settings.fineTuningIncludeAuto') }}
                </label>
                <label class="flex items-center gap-2 text-xs text-gray-300">
                  <input type="checkbox" v-model="fineTuningParams.includeManualRecords" class="rounded bg-darker border-gray-600" />
                  {{ $t('settings.fineTuningIncludeManual') }}
                </label>
              </div>
              <div>
                <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.fineTuningDaysBack') }}</label>
                <input
                  v-model.number="fineTuningParams.daysBack"
                  type="number"
                  min="1"
                  max="365"
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
                />
              </div>
            </div>

            <!-- Model Configuration Section -->
            <div class="space-y-3 pt-3 border-t border-gray-700">
              <h4 class="text-sm font-medium text-gray-300">{{ $t('settings.fineTuningModelConfig') }}</h4>
              <div>
                <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.fineTuningBaseModel') }}</label>
                <select
                  v-model="fineTuningParams.baseModel"
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
                >
                  <option value="" disabled>{{ $t('settings.fineTuningSelectBaseModel') }}</option>
                  <option v-for="model in ollamaModels" :key="model.name" :value="model.name">{{ model.name }}</option>
                </select>
              </div>
              <div>
                <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.fineTuningOutputName') }}</label>
                <input
                  v-model="fineTuningParams.outputModelName"
                  type="text"
                  :placeholder="$t('settings.fineTuningOutputNamePlaceholder')"
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
              </div>
            </div>

            <!-- Advanced Parameters Section -->
            <div class="space-y-3 pt-3 border-t border-gray-700">
              <h4 class="text-sm font-medium text-gray-300">{{ $t('settings.fineTuningAdvancedParams') }}</h4>
              <div>
                <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.fineTuningSystemPrompt') }}</label>
                <textarea
                  v-model="fineTuningParams.systemPrompt"
                  rows="3"
                  :placeholder="$t('settings.fineTuningSystemPromptPlaceholder')"
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y"
                />
              </div>
              <div class="grid grid-cols-3 gap-3">
                <div>
                  <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.fineTuningTemperature') }}</label>
                  <input
                    v-model.number="fineTuningParams.temperature"
                    type="number"
                    min="0"
                    max="2"
                    step="0.1"
                    class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
                  />
                </div>
                <div>
                  <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.fineTuningContext') }}</label>
                  <input
                    v-model.number="fineTuningParams.numCtx"
                    type="number"
                    min="512"
                    step="512"
                    class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
                  />
                </div>
                <div>
                  <label class="text-xs text-gray-300 block mb-1">{{ $t('settings.fineTuningEpochs') }}</label>
                  <input
                    v-model.number="fineTuningParams.epochs"
                    type="number"
                    min="1"
                    max="100"
                    class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 focus:border-primary focus:outline-none"
                  />
                </div>
              </div>
            </div>

            <!-- Export Training Data Section -->
            <div class="pt-3 border-t border-gray-700">
              <button
                @click="exportTrainingData"
                :disabled="isExportingTrainingData"
                type="button"
                class="w-full px-3 py-2 text-sm bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors disabled:opacity-50"
              >
                {{ isExportingTrainingData ? $t('settings.fineTuningExporting') : $t('settings.fineTuningExportData') }}
              </button>
              <p v-if="trainingDataResult" class="text-xs mt-2" :class="trainingDataResult.success ? 'text-green-400' : 'text-red-400'">
                {{ trainingDataResult.message }}
              </p>
            </div>
          </div>
          <div class="px-6 py-4 border-t border-gray-700 flex justify-end gap-3">
            <button
              @click="showFineTuningModal = false"
              class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg text-sm text-gray-200 transition-colors"
            >
              {{ t('settings.cancel') }}
            </button>
            <button
              @click="startFineTuning"
              :disabled="isFineTuning || !fineTuningParams.baseModel || !fineTuningParams.outputModelName"
              class="px-4 py-2 bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-sm font-medium text-white transition-colors"
            >
              {{ isFineTuning ? $t('settings.fineTuningRunning') : $t('settings.fineTuningStart') }}
            </button>
          </div>
        </div>
      </div>

      <div class="px-6 py-4 border-t border-gray-700 flex items-center justify-between gap-3">
        <div class="flex flex-col">
          <span v-if="saveStatus === 'ok'" class="text-green-400 text-xs flex items-center gap-1">
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20"><path d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"/></svg>
            {{ t('settings.saved') }}
          </span>
          <span v-else-if="saveStatus === 'err'" class="text-red-400 text-xs flex items-center gap-1">
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/></svg>
            {{ t('settings.saveFailed') }}
          </span>
          <span v-if="saveError" class="text-xs text-red-400 mt-1">{{ saveError }}</span>
          <span v-else-if="!saveStatus" class="text-xs text-transparent select-none">placeholder</span>
        </div>
        <div class="flex gap-3">
          <button
            @click="$emit('close')"
            class="px-4 py-2 rounded-lg text-sm text-gray-300 hover:bg-gray-700 hover:text-white transition-colors"
          >
            {{ t('settings.cancel') }}
          </button>
          <button
            @click="saveSettings"
            :disabled="isSaving"
            class="px-4 py-2 bg-primary rounded-lg text-sm font-medium text-white hover:bg-blue-600 disabled:opacity-50 transition-colors"
          >
            {{ isSaving ? t('settings.saving') : t('settings.save') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import { writeFile, writeTextFile, readTextFile } from '@tauri-apps/plugin-fs'
import { showError, showSuccess } from '../stores/toast'
import { setLocale } from '../i18n'
import type { Locale } from '../i18n'
import { useI18n } from 'vue-i18n'
import { usePlatform } from '../composables/usePlatform'
import PluginPanel from './PluginPanel.vue'
import TeamPanel from './TeamPanel.vue'
import type { Settings } from '../types/tauri'

interface ModelInfo {
  context_window: number
}

interface ConnectionTestResult {
  success: boolean
  message: string
  latency_ms?: number
}

interface Monitor {
  id: number
  name: string
  width: number
  height: number
  is_primary: boolean
  index?: number
  resolution?: string
}

interface ObsidianVault {
  name: string
  path: string
  is_default?: boolean
}

interface LogseqGraph {
  name: string
  path: string
  is_default?: boolean
}

interface WorkTimeStatus {
  learning_progress: { days_learned: number; min_days_required: number }
  detected_periods?: Array<{ start: number; end: number }>
  current_threshold?: number
  is_work_time?: boolean
}

interface Template {
  id: string
  name: string
  description: string
  content: string | null
}

interface OllamaModel {
  name: string
  size?: string
  modified_at?: string
  size_vram?: number
}

interface OllamaModelsResult {
  success: boolean
  models: OllamaModel[]
  model_details?: OllamaModel[]
  message?: string
}

interface OllamaOperationResult {
  success: boolean
  message: string
}

interface RunningModelsResult {
  success: boolean
  running_models?: RunningModel[]
  message?: string
}

interface TrainingDataResult {
  path: string
  record_count: number
  entries_count?: number
  success?: boolean
  message?: string
}

interface RunningModel {
  name: string
  size_vram?: number
}

const { t, locale } = useI18n()
const { isDesktop } = usePlatform()

const emit = defineEmits<{(e: 'close'): void}>()

const showApiKey = ref(false)
const isSaving = ref(false)
const saveStatus = ref('')
const saveError = ref('')
const isExportingLogs = ref(false)
const exportError = ref('')
const showDefaultPromptModal = ref(false)
const defaultPromptContent = ref('')
const showDefaultSummaryPromptModal = ref(false)
const defaultSummaryPromptContent = ref('')
const showTemplateLibraryModal = ref(false)

// Language switching
function changeLanguage(lang: Locale) {
  setLocale(lang)
  locale.value = lang
}

// API Connection test state
const isTestingConnection = ref(false)
const connectionTestResult = ref<ConnectionTestResult | null>(null)

// Model info state
const isLoadingModelInfo = ref(false)
const analysisModelInfo = ref<ModelInfo | null>(null)
const summaryModelInfo = ref<ModelInfo | null>(null)

// Window whitelist/blacklist tag management
const whitelistTags = ref<string[]>([])
const blacklistTags = ref<string[]>([])
const newWhitelistTag = ref('')
const newBlacklistTag = ref('')

// AI-004: Tag categories
const tagCategoriesText = ref('')
const showDefaultTagCategoriesModal = ref(false)
const defaultTagCategoriesContent = ref<string[]>([])

// Preset templates for summary prompt
const presetTemplates: Template[] = [
  {
    id: 'default',
    name: t('settings.templateDefaultName'),
    description: t('settings.templateDefaultDesc'),
    content: null // Will be loaded from backend
  },
  {
    id: 'concise',
    name: t('settings.templateSimpleName'),
    description: t('settings.templateSimpleDesc'),
    content: `请根据以下今日工作记录，生成简洁的工作摘要。

今日记录：
{records}

要求：
1. 仅列出 3-5 条主要工作项
2. 每项不超过 20 字
3. 格式：• 工作项

请生成摘要：`
  },
  {
    id: 'detailed',
    name: t('settings.templateDetailedName'),
    description: t('settings.templateDetailedDesc'),
    content: `请根据以下今日工作记录，生成详细的工作日报。

今日记录：
{records}

请按以下格式生成日报：

## 📋 今日概览
- 工作时长估算
- 主要工作领域

## ✅ 完成事项
按优先级列出已完成的工作

## 🔄 进行中
正在处理的事项

## 💡 改进建议
基于今日工作的改进建议

## 📌 明日计划
建议的后续事项

请生成日报：`
  }
]

const settings = ref({
  api_base_url: '',
  api_key: '',
  model_name: 'gpt-4o',
  screenshot_interval: 5,
  summary_time: '18:00',
  obsidian_path: '',
  summary_model_name: '',
  analysis_prompt: '',
  summary_prompt: '',
  change_threshold: 3,
  max_silent_minutes: 30,
  summary_title_format: '',
  include_manual_records: true,
  window_whitelist: '[]',
  window_blacklist: '[]',
  use_whitelist_only: false,
  // SMART-002: Auto-adjust silent threshold
  auto_adjust_silent: true,
  silent_adjustment_paused_until: null,
  // SMART-003: Work time auto-detection
  auto_detect_work_time: true,
  use_custom_work_time: false,
  custom_work_time_start: '09:00',
  custom_work_time_end: '18:00',
  learned_work_time: null,
  // SMART-004: Multi-monitor support
  capture_mode: 'primary',
  selected_monitor_index: 0,
  // AI-004: Tag categories
  tag_categories: '',
  // AI-005: Ollama support
  is_ollama: false,
  // DATA-006: Multi Obsidian Vault support
  obsidian_vaults: '[]',
  // INT-002: Logseq graph support
  logseq_graphs: '[]',
  // INT-001: Notion integration
  notion_api_key: null,
  notion_database_id: null,
  // INT-003: GitHub integration
  github_token: null,
  github_repositories: '[]',
  // INT-004: Slack integration
  slack_webhook_url: null
})

// SMART-003: Work time status for learning progress display
const workTimeStatus = ref<WorkTimeStatus | null>(null)

// SMART-004: Monitor settings
const monitors = ref<Monitor[]>([])
const isLoadingMonitors = ref(false)
const monitorError = ref('')
const isScreenshotEnabled = ref(true) // Will be set based on backend capability

// DATA-006: Multi Obsidian Vault support
const vaults = ref<ObsidianVault[]>([])
const newVaultName = ref('')
const newVaultPath = ref('')

// INT-002: Logseq graph support
const graphs = ref<LogseqGraph[]>([])
const newGraphName = ref('')
const newGraphPath = ref('')

// INT-001: Notion integration
const isTestingNotionConnection = ref(false)
const notionConnectionStatus = ref('')

// INT-003: GitHub integration
const isTestingGithubConnection = ref(false)
const githubConnectionStatus = ref('')

// INT-004: Slack integration
const isTestingSlackConnection = ref(false)
const slackConnectionStatus = ref('')

// Computed for GitHub repos (JSON array <-> textarea)
const githubReposText = computed({
  get: () => {
    try {
      const repos = JSON.parse(settings.value.github_repositories || '[]')
      return Array.isArray(repos) ? repos.join('\n') : ''
    } catch {
      return ''
    }
  },
  set: (value) => {
    const repos = value.split('\n').map(r => r.trim()).filter(r => r)
    settings.value.github_repositories = JSON.stringify(repos)
  }
})

const updateGithubRepos = (event: Event) => {
  const target = event.target as HTMLTextAreaElement
  githubReposText.value = target.value
}

const loadSettings = async () => {
  try {
    const loaded = await invoke<Partial<Settings>>('get_settings')
    settings.value = { ...settings.value, ...loaded } as typeof settings.value
    // DATA-006: Parse obsidian_vaults JSON
    try {
      const parsed = JSON.parse(settings.value.obsidian_vaults || '[]')
      vaults.value = Array.isArray(parsed) ? parsed : []
    } catch {
      vaults.value = []
    }
    // Auto-migrate legacy obsidian_path to vaults
    if (vaults.value.length === 0 && settings.value.obsidian_path) {
      vaults.value = [{
        name: 'Default',
        path: settings.value.obsidian_path,
        is_default: true
      }]
    }
    // INT-002: Parse logseq_graphs JSON
    try {
      const parsedGraphs = JSON.parse(settings.value.logseq_graphs || '[]')
      graphs.value = Array.isArray(parsedGraphs) ? parsedGraphs : []
    } catch {
      graphs.value = []
    }
    // Parse window whitelist/blacklist JSON arrays into tag arrays
    try {
      whitelistTags.value = JSON.parse(settings.value.window_whitelist || '[]')
    } catch {
      whitelistTags.value = []
    }
    try {
      blacklistTags.value = JSON.parse(settings.value.window_blacklist || '[]')
    } catch {
      blacklistTags.value = []
    }
    // AI-004: Parse tag categories JSON array into newline-separated text
    try {
      const tagCategories = JSON.parse(settings.value.tag_categories || '[]')
      tagCategoriesText.value = tagCategories.join('\n')
    } catch {
      tagCategoriesText.value = ''
    }
    // SMART-003: Load work time status
    await loadWorkTimeStatus()
    // SMART-004: Load monitors
    await loadMonitors()
  } catch (err) {
    console.error('Failed to load settings:', err)
  }
}

// SMART-003: Load work time status for learning progress display
const loadWorkTimeStatus = async () => {
  try {
    workTimeStatus.value = await invoke<WorkTimeStatus>('get_work_time_status')
  } catch (err) {
    console.error('Failed to load work time status:', err)
    // Don't show error to user - this is optional info
  }
}

// SMART-004: Load monitor list
const loadMonitors = async () => {
  isLoadingMonitors.value = true
  monitorError.value = ''
  try {
    monitors.value = await invoke<Monitor[]>('get_monitors')
  } catch (err) {
    console.error('Failed to load monitors:', err)
    // If get_monitors is not available, screenshot feature is likely disabled
    if (String(err).includes('not found') || String(err).includes('not registered')) {
      isScreenshotEnabled.value = false
    } else {
      monitorError.value = t('settings.monitorLoadFailed', { error: err })
    }
  } finally {
    isLoadingMonitors.value = false
  }
}

// SMART-003: Format work time periods for display
const formatWorkTimePeriods = (periods?: Array<{ start: number; end: number }>) => {
  if (!periods || periods.length === 0) {
    return t('settings.notDetected')
  }
  return periods.map(p => {
    const startHour = String(p.start).padStart(2, '0')
    const endHour = String(p.end).padStart(2, '0')
    return `${startHour}:00-${endHour}:00`
  }).join(', ')
}

// Tag management methods
const addWhitelistTag = () => {
  const tag = newWhitelistTag.value.trim()
  if (tag && !whitelistTags.value.includes(tag)) {
    whitelistTags.value.push(tag)
    newWhitelistTag.value = ''
  }
}

const removeWhitelistTag = (index: number) => {
  whitelistTags.value.splice(index, 1)
}

const addBlacklistTag = () => {
  const tag = newBlacklistTag.value.trim()
  if (tag && !blacklistTags.value.includes(tag)) {
    blacklistTags.value.push(tag)
    newBlacklistTag.value = ''
  }
}

const removeBlacklistTag = (index: number) => {
  blacklistTags.value.splice(index, 1)
}

const validateSettings = (): string | null => {
  // Validate API URL format
  if (settings.value.api_base_url && settings.value.api_base_url.trim()) {
    try {
      new URL(settings.value.api_base_url.trim())
    } catch {
      return 'API Base URL 格式无效，请输入有效的 URL'
    }
  }

  // Validate screenshot interval
  if (settings.value.screenshot_interval < 1 || settings.value.screenshot_interval > 60) {
    return t('settings.screenshotIntervalError')
  }

  // Validate change threshold
  if (settings.value.change_threshold < 1 || settings.value.change_threshold > 20) {
    return t('settings.changeThresholdError')
  }

  // Validate max silent minutes
  if (settings.value.max_silent_minutes < 5 || settings.value.max_silent_minutes > 120) {
    return t('settings.maxSilentTimeError')
  }

  return null
}

// INT-001: Test Notion connection
const testNotionConnection = async () => {
  isTestingNotionConnection.value = true
  notionConnectionStatus.value = ''
  try {
    const result = await invoke('test_notion_connection')
    notionConnectionStatus.value = result ? 'success' : 'failed'
  } catch (error) {
    console.error('Notion connection test failed:', error)
    notionConnectionStatus.value = 'failed'
  } finally {
    isTestingNotionConnection.value = false
  }
}

// INT-003: Test GitHub connection
const testGithubConnection = async () => {
  isTestingGithubConnection.value = true
  githubConnectionStatus.value = ''
  try {
    const result = await invoke('test_github_connection')
    githubConnectionStatus.value = result ? 'success' : 'failed'
  } catch (error) {
    console.error('GitHub connection test failed:', error)
    githubConnectionStatus.value = 'failed'
  } finally {
    isTestingGithubConnection.value = false
  }
}

// INT-004: Test Slack connection
const testSlackConnection = async () => {
  isTestingSlackConnection.value = true
  slackConnectionStatus.value = ''
  try {
    const result = await invoke('test_slack_connection')
    slackConnectionStatus.value = result ? 'success' : 'failed'
  } catch (error) {
    console.error('Slack connection test failed:', error)
    slackConnectionStatus.value = 'failed'
  } finally {
    isTestingSlackConnection.value = false
  }
}

const saveSettings = async () => {
  // Validate settings first
  const validationError = validateSettings()
  if (validationError) {
    saveStatus.value = 'err'
    saveError.value = validationError
    return
  }

  isSaving.value = true
  saveStatus.value = ''
  saveError.value = ''
  try {
    // Convert tag arrays to JSON strings before saving
    const settingsToSave = {
      ...settings.value,
      window_whitelist: JSON.stringify(whitelistTags.value),
      window_blacklist: JSON.stringify(blacklistTags.value),
      // AI-004: Convert tag categories text to JSON array
      tag_categories: JSON.stringify(
        tagCategoriesText.value
          .split('\n')
          .map(t => t.trim())
          .filter(t => t.length > 0)
      ),
      // DATA-006: Serialize vaults to JSON and sync legacy obsidian_path
      obsidian_vaults: JSON.stringify(vaults.value),
      obsidian_path: vaults.value.find(v => v.is_default)?.path || vaults.value[0]?.path || settings.value.obsidian_path || '',
      // INT-002: Serialize Logseq graphs to JSON
      logseq_graphs: JSON.stringify(graphs.value)
    }
    await invoke('save_settings', { settings: settingsToSave })
    saveStatus.value = 'ok'
    showSuccess(t('settings.settingsSaved'))
    setTimeout(() => emit('close'), 800)
  } catch (err) {
    console.error('Failed to save settings:', err)
    saveStatus.value = 'err'
    saveError.value = String(err)
    showError(err)
  } finally {
    isSaving.value = false
  }
}

// DATA-006: Vault management methods
const addVault = () => {
  const name = newVaultName.value.trim()
  const path = newVaultPath.value.trim()
  if (!name || !path) return
  const isFirst = vaults.value.length === 0
  vaults.value.push({ name, path, is_default: isFirst })
  newVaultName.value = ''
  newVaultPath.value = ''
}

const removeVault = (index: number) => {
  const wasDefault = vaults.value[index].is_default
  vaults.value.splice(index, 1)
  if (wasDefault && vaults.value.length > 0) {
    vaults.value[0].is_default = true
  }
}

const setDefaultVault = (index: number) => {
  vaults.value.forEach((v, i) => { v.is_default = i === index })
}

// INT-002: Logseq graph management methods
const addGraph = () => {
  const name = newGraphName.value.trim()
  const path = newGraphPath.value.trim()
  if (!name || !path) return
  const isFirst = graphs.value.length === 0
  graphs.value.push({ name, path, is_default: isFirst })
  newGraphName.value = ''
  newGraphPath.value = ''
}

const removeGraph = (index: number) => {
  const wasDefault = graphs.value[index].is_default
  graphs.value.splice(index, 1)
  if (wasDefault && graphs.value.length > 0) {
    graphs.value[0].is_default = true
  }
}

const setDefaultGraph = (index: number) => {
  graphs.value.forEach((g, i) => { g.is_default = i === index })
}

const exportLogs = async () => {
  isExportingLogs.value = true
  exportError.value = ''

  try {
    // Get log content
    const logContent = await invoke<string>('get_logs_for_export')

    // Open save dialog
    const filePath = await save({
      defaultPath: `daily-logger-${new Date().toISOString().slice(0, 10)}.log`,
      filters: [
        { name: 'Log Files', extensions: ['log'] },
        { name: 'Text Files', extensions: ['txt'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    })

    if (filePath) {
      // Write the log content to the selected file
      await writeTextFile(filePath, logContent)
      showSuccess(t('settings.logsExportSuccess'))
    }
  } catch (err) {
    console.error('Failed to export logs:', err)
    if (err !== 'Log file does not exist') {
      exportError.value = t('settings.exportFailedMsg', { error: err })
      showError(err)
    } else {
      exportError.value = t('settings.logFileNotExist')
    }
  } finally {
    isExportingLogs.value = false
  }
}

const showDefaultPrompt = async () => {
  try {
    defaultPromptContent.value = await invoke('get_default_analysis_prompt')
    showDefaultPromptModal.value = true
  } catch (err) {
    console.error('Failed to get default prompt:', err)
    showError(err)
  }
}

const resetPrompt = () => {
  settings.value.analysis_prompt = ''
  showSuccess(t('settings.resetPromptSuccess'))
}

// AI-004: Tag categories methods
const showDefaultTagCategories = async () => {
  try {
    defaultTagCategoriesContent.value = await invoke('get_default_tag_categories')
    showDefaultTagCategoriesModal.value = true
  } catch (err) {
    console.error('Failed to get default tag categories:', err)
    showError(err)
  }
}

const resetTagCategories = () => {
  tagCategoriesText.value = ''
  showSuccess(t('settings.resetTagCategoriesSuccess'))
}

// API Connection test
const testConnection = async () => {
  if (!settings.value.api_base_url || !settings.value.model_name) {
    showError(t('settings.apiBaseUrlRequired'))
    return
  }

  // For non-Ollama endpoints, API Key is required
  const isOllama = isOllamaEndpoint(settings.value.api_base_url)
  if (!isOllama && !settings.value.api_key) {
    showError(t('settings.apiKeyRequired'))
    return
  }

  isTestingConnection.value = true
  connectionTestResult.value = null

  try {
    const result = await invoke<ConnectionTestResult>('test_api_connection_with_ollama', {
      apiBaseUrl: settings.value.api_base_url,
      apiKey: settings.value.api_key || null,
      modelName: settings.value.model_name
    })
    connectionTestResult.value = result
    if (result.success) {
      showSuccess(t('settings.connectionSuccess', { latency: result.latency_ms }))
    } else {
      showError(result.message)
    }
  } catch (err) {
    console.error('Failed to test connection:', err)
    connectionTestResult.value = { success: false, message: String(err) }
    showError(err)
  } finally {
    isTestingConnection.value = false
  }
}

// Get model info
const getModelInfo = async (type: 'analysis' | 'summary') => {
  const modelName = type === 'analysis' ? settings.value.model_name : settings.value.summary_model_name
  if (!modelName) {
    showError(t('settings.modelNameRequired'))
    return
  }

  isLoadingModelInfo.value = true

  try {
    const result = await invoke<ModelInfo | { error: string; context_window?: number }>('get_model_info', {
      apiBaseUrl: settings.value.api_base_url,
      apiKey: settings.value.api_key,
      modelName: modelName
    })

    if (type === 'analysis') {
      analysisModelInfo.value = result as ModelInfo
    } else {
      summaryModelInfo.value = result as ModelInfo
    }

    if ('error' in result && result.error) {
      showError(result.error)
    } else if ('context_window' in result && result.context_window) {
      showSuccess(t('settings.modelContextWindow', { model: modelName, size: result.context_window / 1000 }))
    } else {
      showSuccess(t('settings.modelInfoUnavailable'))
    }
  } catch (err) {
    console.error('Failed to get model info:', err)
    showError(err)
  } finally {
    isLoadingModelInfo.value = false
  }
}

// AI-005: Ollama support
const ollamaModels = ref<OllamaModel[]>([])
const ollamaModelDetails = ref<OllamaModel[]>([])
const isLoadingOllamaModels = ref(false)
const ollamaModelError = ref('')
const pullModelName = ref('')
const pullModelQuantization = ref('')
const isPullingModel = ref(false)
const isDeletingModel = ref('')

// Running models state
const runningModels = ref<RunningModel[]>([])
const isLoadingRunningModels = ref(false)

// Create custom model state
const showCreateModelModal = ref(false)
const isCreatingModel = ref(false)
const createModelParams = ref({
  name: '',
  from: '',
  system: '',
  temperature: null,
  num_ctx: null,
  quantize: ''
})

// Copy model state
const showCopyModelModal = ref(false)
const isCopyingModel = ref(false)
const copyModelSource = ref('')
const copyModelDestination = ref('')

// Fine-tuning state
const showFineTuningModal = ref(false)
const isFineTuning = ref(false)
const isExportingTrainingData = ref(false)
interface TrainingDataResult {
  path: string
  record_count: number
}
const trainingDataResult = ref<TrainingDataResult | null>(null)
const fineTuningParams = ref({
  baseModel: '',
  outputModelName: '',
  includeAutoRecords: true,
  includeManualRecords: true,
  daysBack: 30,
  systemPrompt: '',
  temperature: 0.7,
  numCtx: 4096,
  epochs: 3
})

// Check if the current endpoint is an Ollama endpoint
const isOllamaEndpoint = (url: string): boolean => {
  if (!url) return false
  const urlLower = url.toLowerCase()
  return urlLower.includes('localhost:11434') ||
         urlLower.includes('127.0.0.1:11434') ||
         urlLower.includes(':11434/v1') ||
         urlLower.includes(':11434/')
}

// Computed property for Ollama status
const isOllama = computed(() => isOllamaEndpoint(settings.value.api_base_url))

// Fetch available models from Ollama
const fetchOllamaModels = async () => {
  if (!settings.value.api_base_url) {
    showError(t('settings.apiBaseUrlRequired'))
    return
  }

  isLoadingOllamaModels.value = true
  ollamaModelError.value = ''

  try {
    const result = await invoke<OllamaModelsResult>('get_ollama_models', {
      baseUrl: settings.value.api_base_url
    })

    if (result.success) {
      ollamaModels.value = result.models
      ollamaModelDetails.value = result.model_details || []
      if (result.models.length === 0) {
        ollamaModelError.value = t('settings.ollamaModelsNotFound')
      } else {
        showSuccess(t('settings.ollamaModelsFound', { count: result.models.length }))
      }
    } else {
      ollamaModelError.value = result.message || ''
      showError(result.message || '')
    }
  } catch (err) {
    console.error('Failed to fetch Ollama models:', err)
    ollamaModelError.value = String(err)
    showError(err)
  } finally {
    isLoadingOllamaModels.value = false
  }
}

// Select an Ollama model
const selectOllamaModel = (modelName: string): void => {
  settings.value.model_name = modelName
}

// Pull a model from Ollama registry
const pullModel = async () => {
  if (!pullModelName.value.trim()) {
    showError(t('settings.modelNameRequired'))
    return
  }

  if (!settings.value.api_base_url) {
    showError(t('settings.apiBaseUrlRequired'))
    return
  }

  isPullingModel.value = true
  ollamaModelError.value = ''

  try {
    const result = await invoke<OllamaOperationResult>('pull_ollama_model', {
      baseUrl: settings.value.api_base_url,
      modelName: pullModelName.value.trim(),
      quantization: pullModelQuantization.value.trim() || null
    })

    if (result.success) {
      showSuccess(result.message)
      pullModelName.value = ''
      pullModelQuantization.value = ''
      // Refresh the model list
      await fetchOllamaModels()
    } else {
      showError(result.message)
    }
  } catch (err) {
    console.error('Failed to pull model:', err)
    showError(err)
  } finally {
    isPullingModel.value = false
  }
}

// Delete a model from Ollama
const deleteModel = async (modelName: string) => {
  if (!confirm(t('settings.confirmDeleteModel', { model: modelName }))) {
    return
  }

  isDeletingModel.value = modelName

  try {
    const result = await invoke<OllamaOperationResult>('delete_ollama_model', {
      baseUrl: settings.value.api_base_url,
      modelName: modelName
    })

    if (result.success) {
      showSuccess(result.message)
      // Refresh the model list
      await fetchOllamaModels()
      // Clear model selection if deleted model was selected
      if (settings.value.model_name === modelName) {
        settings.value.model_name = ''
      }
    } else {
      showError(result.message)
    }
  } catch (err) {
    console.error('Failed to delete model:', err)
    showError(err)
  } finally {
    isDeletingModel.value = ''
  }
}

// Format model size to human readable format
const formatModelSize = (bytes: number | undefined) => {
  if (!bytes) return ''
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}

// Get model size by name
const getModelSize = (modelName: string) => {
  const detail = ollamaModelDetails.value.find(d => d.name === modelName)
  // size is a string, return it directly or use size_vram for formatting
  if (detail?.size) return detail.size
  if (detail?.size_vram) return formatModelSize(detail.size_vram)
  return ''
}

// Fetch running models from Ollama
const fetchRunningModels = async () => {
  if (!settings.value.api_base_url) {
    return
  }

  isLoadingRunningModels.value = true

  try {
    const result = await invoke<RunningModelsResult>('get_running_models', {
      baseUrl: settings.value.api_base_url
    })

    if (result.success) {
      runningModels.value = result.running_models || []
    } else {
      runningModels.value = []
      console.warn('Failed to fetch running models:', result.message)
    }
  } catch (err) {
    console.error('Failed to fetch running models:', err)
    runningModels.value = []
  } finally {
    isLoadingRunningModels.value = false
  }
}

// Create a custom model from a base model
const createCustomModel = async () => {
  if (!createModelParams.value.name || !createModelParams.value.from) {
    return
  }

  if (!settings.value.api_base_url) {
    showError(t('settings.apiBaseUrlRequired'))
    return
  }

  isCreatingModel.value = true

  try {
    // Build parameters object (only include non-null values)
    const parameters: { temperature?: number; num_ctx?: number } = {}
    if (createModelParams.value.temperature !== null) {
      parameters.temperature = createModelParams.value.temperature
    }
    if (createModelParams.value.num_ctx !== null) {
      parameters.num_ctx = createModelParams.value.num_ctx
    }

    const result = await invoke<OllamaOperationResult & { model_name?: string }>('create_ollama_model', {
      baseUrl: settings.value.api_base_url,
      params: {
        name: createModelParams.value.name.trim(),
        from: createModelParams.value.from,
        system: createModelParams.value.system || null,
        parameters: Object.keys(parameters).length > 0 ? parameters : null,
        quantize: createModelParams.value.quantize || null
      }
    })

    if (result.success) {
      showSuccess(t('settings.createModelSuccess', { model: result.model_name }))
      // Reset form
      createModelParams.value = {
        name: '',
        from: '',
        system: '',
        temperature: null,
        num_ctx: null,
        quantize: ''
      }
      showCreateModelModal.value = false
      // Refresh the model list
      await fetchOllamaModels()
    } else {
      showError(t('settings.createModelFailed', { error: result.message }))
    }
  } catch (err) {
    console.error('Failed to create model:', err)
    showError(t('settings.createModelFailed', { error: String(err) }))
  } finally {
    isCreatingModel.value = false
  }
}

// Open copy model modal for a specific model
const openCopyModelModal = (modelName: string) => {
  copyModelSource.value = modelName
  copyModelDestination.value = ''
  showCopyModelModal.value = true
}

// Copy a model from Ollama
const copyModel = async () => {
  if (!copyModelSource.value || !copyModelDestination.value.trim()) {
    return
  }

  if (!settings.value.api_base_url) {
    showError(t('settings.apiBaseUrlRequired'))
    return
  }

  isCopyingModel.value = true

  try {
    const result = await invoke<OllamaOperationResult>('copy_ollama_model', {
      baseUrl: settings.value.api_base_url,
      source: copyModelSource.value,
      destination: copyModelDestination.value.trim()
    })

    if (result.success) {
      showSuccess(t('settings.copyModelSuccess', {
        source: copyModelSource.value,
        destination: copyModelDestination.value.trim()
      }))
      showCopyModelModal.value = false
      copyModelSource.value = ''
      copyModelDestination.value = ''
      // Refresh the model list
      await fetchOllamaModels()
    } else {
      showError(t('settings.copyModelFailed', { error: result.message }))
    }
  } catch (err) {
    console.error('Failed to copy model:', err)
    showError(t('settings.copyModelFailed', { error: String(err) }))
  } finally {
    isCopyingModel.value = false
  }
}

// FUTURE-003: Fine-tuning functions
const exportTrainingData = async () => {
  if (!fineTuningParams.value.includeAutoRecords && !fineTuningParams.value.includeManualRecords) {
    showError(t('settings.fineTuningSelectRecordType'))
    return
  }

  isExportingTrainingData.value = true
  trainingDataResult.value = null

  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const filePath = await save({
      defaultPath: `training-data-${new Date().toISOString().slice(0, 10)}.jsonl`,
      filters: [{ name: 'JSONL', extensions: ['jsonl'] }]
    })

    if (filePath) {
      const result = await invoke<TrainingDataResult>('prepare_training_data', {
        outputPath: filePath,
        includeAutoRecords: fineTuningParams.value.includeAutoRecords,
        includeManualRecords: fineTuningParams.value.includeManualRecords,
        daysBack: fineTuningParams.value.daysBack
      })
      trainingDataResult.value = result
      if (result.success) {
        showSuccess(t('settings.fineTuningExportSuccess', { count: result.entries_count }))
      }
    }
  } catch (err) {
    console.error('Failed to export training data:', err)
    trainingDataResult.value = { path: '', record_count: 0, success: false, message: String(err) }
    showError(err)
  } finally {
    isExportingTrainingData.value = false
  }
}

const startFineTuning = async () => {
  if (!fineTuningParams.value.baseModel) {
    showError(t('settings.fineTuningSelectBaseModel'))
    return
  }

  if (!fineTuningParams.value.outputModelName.trim()) {
    showError(t('settings.fineTuningOutputNameRequired'))
    return
  }

  if (!settings.value.api_base_url) {
    showError(t('settings.apiBaseUrlRequired'))
    return
  }

  isFineTuning.value = true

  try {
    const result = await invoke<OllamaOperationResult & { model_name?: string }>('start_fine_tuning', {
      baseUrl: settings.value.api_base_url,
      config: {
        base_model: fineTuningParams.value.baseModel,
        output_model_name: fineTuningParams.value.outputModelName.trim(),
        epochs: fineTuningParams.value.epochs,
        system_prompt: fineTuningParams.value.systemPrompt || null,
        temperature: fineTuningParams.value.temperature,
        num_ctx: fineTuningParams.value.numCtx
      }
    })

    if (result.success) {
      showSuccess(t('settings.fineTuningSuccess', { model: result.model_name }))
      showFineTuningModal.value = false
      // Refresh the model list
      await fetchOllamaModels()
    } else {
      showError(t('settings.fineTuningFailed', { error: result.message }))
    }
  } catch (err) {
    console.error('Failed to start fine-tuning:', err)
    showError(t('settings.fineTuningFailed', { error: String(err) }))
  } finally {
    isFineTuning.value = false
  }
}

// Summary Prompt functions
const showDefaultSummaryPrompt = async () => {
  try {
    defaultSummaryPromptContent.value = await invoke('get_default_summary_prompt')
    showDefaultSummaryPromptModal.value = true
  } catch (err) {
    console.error('Failed to get default summary prompt:', err)
    showError(err)
  }
}

const resetSummaryPrompt = () => {
  settings.value.summary_prompt = ''
  showSuccess(t('settings.resetPromptSuccess'))
}

// Template Library functions
const showTemplateLibrary = async () => {
  // Load default template content from backend
  try {
    const defaultPrompt = await invoke<string | null>('get_default_summary_prompt')
    presetTemplates[0].content = defaultPrompt
  } catch (err) {
    console.error('Failed to get default summary prompt:', err)
  }
  showTemplateLibraryModal.value = true
}

const applyTemplate = (template: Template) => {
  if (template.content) {
    settings.value.summary_prompt = template.content
    showTemplateLibraryModal.value = false
    showSuccess(t('settings.templateApplied', { name: template.name }))
  } else {
    showError(t('settings.templateContentEmpty'))
  }
}

// Import/Export functions
const exportTemplate = async () => {
  const currentPrompt = settings.value.summary_prompt || ''
  if (!currentPrompt.trim()) {
    showError(t('settings.currentPromptEmpty'))
    return
  }

  try {
    const templateData = {
      version: '1.0',
      name: t('settings.myReportTemplateName'),
      description: t('settings.myReportTemplateDesc'),
      content: currentPrompt,
      createdAt: new Date().toISOString()
    }

    const filePath = await save({
      defaultPath: `summary-template-${new Date().toISOString().slice(0, 10)}.json`,
      filters: [
        { name: 'JSON Files', extensions: ['json'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    })

    if (filePath) {
      await writeTextFile(filePath, JSON.stringify(templateData, null, 2))
      showSuccess(t('settings.templateExportSuccess'))
    }
  } catch (err) {
    console.error('Failed to export template:', err)
    showError(t('settings.exportFailedMsg', { error: err }))
  }
}

const importTemplate = async () => {
  try {
    const filePath = await open({
      multiple: false,
      filters: [
        { name: 'JSON Files', extensions: ['json'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    })

    if (!filePath) {
      return
    }

    const content = await readTextFile(filePath)
    const templateData = JSON.parse(content)

    // Validate template format
    if (!templateData.content || typeof templateData.content !== 'string') {
      showError(t('settings.invalidTemplateFile'))
      return
    }

    // Check for {records} placeholder
    if (!templateData.content.includes('{records}')) {
      showError(t('settings.templateMissingPlaceholder'))
      return
    }

    settings.value.summary_prompt = templateData.content
    showSuccess(t('settings.importSuccess', { name: templateData.name || t('settings.unnamedTemplate') }))
  } catch (err) {
    console.error('Failed to import template:', err)
    if (err instanceof SyntaxError) {
      showError(t('settings.importFailed'))
    } else {
      showError(t('settings.importFailedMsg', { error: err }))
    }
  }
}

onMounted(() => {
  loadSettings()
})
</script>
