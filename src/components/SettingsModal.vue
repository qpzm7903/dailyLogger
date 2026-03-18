<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="$emit('close')">
    <div class="bg-dark rounded-2xl w-[500px] max-h-[80vh] overflow-y-auto border border-gray-700">
      <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
        <h2 class="text-lg font-semibold">设置</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">✕</button>
      </div>
      
      <div class="p-6 space-y-6">
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">API 配置</h3>
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
                Ollama 用户请填写 http://localhost:11434/v1
              </span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">
                API Key
                <span v-if="isOllama" class="text-gray-500">(Ollama 可留空)</span>
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
                  :title="showApiKey ? '隐藏' : '显示'"
                >{{ showApiKey ? '隐藏' : '显示' }}</button>
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
                  {{ isTestingConnection ? '测试中...' : '测试连接' }}
                </button>
                <!-- Ollama model fetch button -->
                <button
                  v-if="isOllama"
                  @click="fetchOllamaModels"
                  :disabled="isLoadingOllamaModels || !settings.api_base_url"
                  class="px-3 py-1.5 text-sm bg-purple-700 hover:bg-purple-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
                >
                  {{ isLoadingOllamaModels ? '获取中...' : '获取模型列表' }}
                </button>
              </div>
              <span v-if="connectionTestResult" :class="connectionTestResult.success ? 'text-green-400' : 'text-red-400'" class="ml-2 text-xs">
                {{ connectionTestResult.message }}
                <span v-if="connectionTestResult.latency_ms">({{ connectionTestResult.latency_ms }}ms)</span>
              </span>
              <!-- Ollama model list -->
              <div v-if="isOllama && ollamaModels.length > 0" class="mt-3">
                <label class="text-xs text-gray-300 block mb-1">选择模型</label>
                <div class="flex flex-wrap gap-2">
                  <button
                    v-for="model in ollamaModels"
                    :key="model"
                    @click="selectOllamaModel(model)"
                    class="px-2 py-1 text-xs rounded border transition-colors"
                    :class="settings.model_name === model ? 'bg-primary border-primary text-white' : 'bg-darker border-gray-600 text-gray-300 hover:border-primary'"
                  >
                    {{ model }}
                  </button>
                </div>
              </div>
              <p v-if="ollamaModelError" class="text-xs text-red-400 mt-1">{{ ollamaModelError }}</p>
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">截图分析 (Vision)</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">分析模型</label>
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
                  title="查看模型上下文窗口"
                >ℹ️</button>
              </div>
              <span v-if="analysisModelInfo?.context_window" class="text-xs text-gray-500 mt-1 block">
                上下文窗口: {{ analysisModelInfo.context_window / 1000 }}K tokens
              </span>
              <span v-else class="text-xs text-gray-500 mt-1 block">需要支持 Vision 能力的模型</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">分析 Prompt</label>
              <textarea
                v-model="settings.analysis_prompt"
                rows="4"
                placeholder="留空使用默认 Prompt"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y"
              />
              <div class="flex gap-3 mt-2">
                <button
                  type="button"
                  @click="showDefaultPrompt"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  查看默认
                </button>
                <button
                  type="button"
                  @click="resetPrompt"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  重置为默认
                </button>
              </div>
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">日报生成</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">日报标题格式</label>
              <input
                v-model="settings.summary_title_format"
                type="text"
                placeholder="工作日报 - {date}"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-500 mt-1 block">使用 {date} 作为日期占位符，留空使用默认格式</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">日报模型</label>
              <div class="flex items-center gap-2">
                <input
                  v-model="settings.summary_model_name"
                  type="text"
                  placeholder="留空则使用分析模型"
                  class="flex-1 bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
                <button
                  @click="getModelInfo('summary')"
                  :disabled="isLoadingModelInfo || !settings.summary_model_name"
                  type="button"
                  class="text-gray-400 hover:text-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors px-2"
                  title="查看模型上下文窗口"
                >ℹ️</button>
              </div>
              <span v-if="summaryModelInfo?.context_window" class="text-xs text-gray-500 mt-1 block">
                上下文窗口: {{ summaryModelInfo.context_window / 1000 }}K tokens
              </span>
              <span v-else class="text-xs text-gray-500 mt-1 block">纯文本模型即可，不需要 Vision</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">日报 Prompt</label>
              <textarea
                v-model="settings.summary_prompt"
                rows="4"
                placeholder="留空使用默认 Prompt。用 {records} 表示今日记录的插入位置"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y"
              />
              <div class="flex gap-3 mt-2">
                <button
                  type="button"
                  @click="showDefaultSummaryPrompt"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  查看默认
                </button>
                <button
                  type="button"
                  @click="resetSummaryPrompt"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  重置为默认
                </button>
                <button
                  type="button"
                  @click="showTemplateLibrary"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  模板库
                </button>
                <button
                  type="button"
                  @click="exportTemplate"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  导出模板
                </button>
                <button
                  type="button"
                  @click="importTemplate"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  导入模板
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
                包含闪念胶囊记录
              </label>
              <span class="text-xs text-gray-500">（取消勾选则仅使用自动截图分析）</span>
            </div>
          </div>
        </div>

        <!-- AI-004: 标签分类配置 -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">标签分类</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">自定义标签分类</label>
              <textarea
                v-model="tagCategoriesText"
                rows="4"
                placeholder="每行一个标签，留空使用默认分类"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none resize-y font-mono"
              />
              <span class="text-xs text-gray-500 mt-1 block">AI 分析时将从这些标签中选择最匹配的分类</span>
              <div class="flex gap-3 mt-2">
                <button
                  type="button"
                  @click="showDefaultTagCategories"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  查看默认标签
                </button>
                <button
                  type="button"
                  @click="resetTagCategories"
                  class="text-xs text-gray-400 hover:text-primary transition-colors"
                >
                  重置为默认
                </button>
              </div>
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">时间策略</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">截图间隔 (分钟)</label>
              <input
                v-model.number="settings.screenshot_interval"
                type="number"
                min="1"
                max="60"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">每日总结时间</label>
              <input
                v-model="settings.summary_time"
                type="time"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">智能去重</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">变化阈值 (%)</label>
              <input
                v-model.number="settings.change_threshold"
                type="number"
                min="1"
                max="20"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-500 mt-1 block">屏幕变化低于此比例时跳过截图，避免重复记录</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">最大静默时间 (分钟)</label>
              <input
                v-model.number="settings.max_silent_minutes"
                type="number"
                min="5"
                max="120"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
              />
              <span class="text-xs text-gray-500 mt-1 block">即使屏幕无变化，超过此时间也会强制记录一次</span>
            </div>
          </div>
        </div>

        <!-- SMART-002: 智能静默阈值调整 -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">静默阈值智能调整</h3>
          <div class="space-y-3">
            <div class="flex items-center gap-2">
              <input
                v-model="settings.auto_adjust_silent"
                type="checkbox"
                id="auto_adjust_silent"
                class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
              />
              <label for="auto_adjust_silent" class="text-xs text-gray-300 cursor-pointer">
                自动调整静默阈值
              </label>
            </div>
            <span class="text-xs text-gray-500 block">
              根据工作模式自动调整：深度工作时提高阈值，活跃工作时降低阈值
            </span>
            <div v-if="!settings.auto_adjust_silent" class="bg-darker rounded-lg p-3 border border-gray-700">
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-gray-400">手动模式已启用</span>
                <span class="text-xs text-primary">{{ settings.max_silent_minutes }} 分钟</span>
              </div>
              <span class="text-xs text-gray-500">
                关闭自动调整后，系统将使用您设定的固定阈值
              </span>
            </div>
            <div v-else class="bg-darker rounded-lg p-3 border border-gray-700">
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-gray-400">学习状态</span>
                <span class="text-xs text-green-400">自动学习中</span>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-xs text-gray-400">当前阈值</span>
                <span class="text-xs text-primary">{{ settings.max_silent_minutes }} 分钟</span>
              </div>
              <span class="text-xs text-gray-500 mt-2 block">
                系统每小时自动评估并调整阈值（范围: 10-60 分钟）
              </span>
            </div>
          </div>
        </div>

        <!-- SMART-003: 工作时间自动识别 -->
        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">工作时间自动识别</h3>
          <div class="space-y-3">
            <div class="flex items-center gap-2">
              <input
                v-model="settings.auto_detect_work_time"
                type="checkbox"
                id="auto_detect_work_time"
                class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
              />
              <label for="auto_detect_work_time" class="text-xs text-gray-300 cursor-pointer">
                自动识别工作时间
              </label>
            </div>
            <span class="text-xs text-gray-500 block">
              根据截图活动模式自动学习工作时间，非工作时间自动暂停捕获
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
                使用自定义工作时间
              </label>
            </div>

            <!-- Custom work time inputs -->
            <div v-if="settings.auto_detect_work_time && settings.use_custom_work_time" class="grid grid-cols-2 gap-3 pt-2">
              <div>
                <label class="text-xs text-gray-300 block mb-1">开始时间</label>
                <input
                  v-model="settings.custom_work_time_start"
                  type="time"
                  class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                />
              </div>
              <div>
                <label class="text-xs text-gray-300 block mb-1">结束时间</label>
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
                <span class="text-xs text-gray-400">学习状态</span>
                <span v-if="workTimeStatus" class="text-xs" :class="workTimeStatus.is_work_time ? 'text-green-400' : 'text-yellow-400'">
                  {{ workTimeStatus.is_work_time ? '工作中' : '非工作时间' }}
                </span>
              </div>
              <div v-if="workTimeStatus" class="space-y-1">
                <div class="flex items-center justify-between">
                  <span class="text-xs text-gray-400">已学习天数</span>
                  <span class="text-xs text-primary">{{ workTimeStatus.learning_progress.days_learned }} / {{ workTimeStatus.learning_progress.min_days_required }} 天</span>
                </div>
                <div v-if="workTimeStatus.learning_progress.days_learned >= workTimeStatus.learning_progress.min_days_required" class="flex items-center justify-between">
                  <span class="text-xs text-gray-400">识别的工作时间</span>
                  <span class="text-xs text-gray-300">{{ formatWorkTimePeriods(workTimeStatus.detected_periods) }}</span>
                </div>
                <div v-else class="mt-2">
                  <div class="w-full bg-gray-700 rounded-full h-1.5">
                    <div
                      class="bg-primary h-1.5 rounded-full transition-all"
                      :style="{ width: `${Math.min(100, (workTimeStatus.learning_progress.days_learned / workTimeStatus.learning_progress.min_days_required) * 100)}%` }"
                    ></div>
                  </div>
                  <span class="text-xs text-gray-500 mt-1 block">继续使用以学习您的工作模式</span>
                </div>
              </div>
            </div>

            <!-- Disabled info -->
            <div v-if="!settings.auto_detect_work_time" class="bg-darker rounded-lg p-3 border border-gray-700">
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-gray-400">工作时间检测</span>
                <span class="text-xs text-gray-500">已关闭</span>
              </div>
              <span class="text-xs text-gray-500">
                关闭后，系统将在全天候进行截图捕获
              </span>
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">窗口过滤</h3>
          <div class="space-y-3">
            <div>
              <label class="text-xs text-gray-300 block mb-1">窗口白名单</label>
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
                placeholder="输入应用名后按 Enter 添加白名单"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                @keyup.enter="addWhitelistTag"
              />
              <span class="text-xs text-gray-500 mt-1 block">匹配窗口标题或进程名，支持部分匹配</span>
            </div>
            <div>
              <label class="text-xs text-gray-300 block mb-1">窗口黑名单</label>
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
                placeholder="输入应用名后按 Enter 添加黑名单"
                class="w-full bg-darker border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none"
                @keyup.enter="addBlacklistTag"
              />
              <span class="text-xs text-gray-500 mt-1 block">匹配窗口标题或进程名，支持部分匹配</span>
            </div>
            <div class="flex items-center gap-2 pt-1">
              <input
                v-model="settings.use_whitelist_only"
                type="checkbox"
                id="use_whitelist_only"
                class="w-4 h-4 rounded border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0 cursor-pointer"
              />
              <label for="use_whitelist_only" class="text-xs text-gray-300 cursor-pointer">
                仅捕获白名单应用
              </label>
              <span class="text-xs text-gray-500">（启用后仅捕获白名单中的应用）</span>
            </div>
          </div>
        </div>

        <!-- SMART-004: 显示器设置 -->
        <div v-if="isScreenshotEnabled">
          <h3 class="text-sm font-medium text-gray-300 mb-3">显示器设置</h3>
          <div class="space-y-3">
            <!-- 多显示器时显示捕获模式选择 -->
            <div v-if="monitors?.length > 1" class="space-y-2">
              <label class="text-xs text-gray-300 block mb-1">捕获模式</label>
              <div class="flex flex-wrap gap-4">
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    v-model="settings.capture_mode"
                    value="primary"
                    class="w-4 h-4 border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0"
                  />
                  <span class="text-sm text-gray-300">主显示器</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    v-model="settings.capture_mode"
                    value="secondary"
                    class="w-4 h-4 border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0"
                  />
                  <span class="text-sm text-gray-300">副显示器</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    v-model="settings.capture_mode"
                    value="all"
                    class="w-4 h-4 border-gray-600 bg-darker text-primary focus:ring-primary focus:ring-offset-0"
                  />
                  <span class="text-sm text-gray-300">全部拼接</span>
                </label>
              </div>
            </div>

            <!-- 显示器列表 -->
            <div v-if="monitors?.length > 1" class="space-y-1">
              <label class="text-xs text-gray-300 block mb-1">已连接显示器</label>
              <div
                v-for="m in monitors"
                :key="m.index"
                class="flex items-center gap-2 text-sm bg-darker rounded-lg px-3 py-2 border border-gray-700"
              >
                <span class="text-gray-300">{{ m.name }}</span>
                <span class="text-gray-500">{{ m.resolution }}</span>
                <span v-if="m.is_primary" class="text-xs bg-primary/20 text-primary px-1.5 py-0.5 rounded">主</span>
                <!-- 副显示器模式下可选择 -->
                <button
                  v-if="settings.capture_mode === 'secondary' && !m.is_primary"
                  type="button"
                  @click="settings.selected_monitor_index = m.index"
                  :class="[
                    'ml-auto text-xs px-2 py-1 rounded transition-colors',
                    settings.selected_monitor_index === m.index
                      ? 'bg-primary text-white'
                      : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                  ]"
                >
                  {{ settings.selected_monitor_index === m.index ? '已选择' : '选择' }}
                </button>
              </div>
            </div>

            <!-- 单显示器提示 -->
            <div v-if="monitors?.length === 1" class="bg-darker rounded-lg p-3 border border-gray-700">
              <div class="flex items-center justify-between mb-1">
                <span class="text-xs text-gray-400">当前显示器</span>
                <span class="text-xs text-gray-300">{{ monitors[0]?.name || '未知' }}</span>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-xs text-gray-400">分辨率</span>
                <span class="text-xs text-gray-300">{{ monitors[0]?.resolution || '未知' }}</span>
              </div>
            </div>

            <!-- 加载中或错误状态 -->
            <div v-if="isLoadingMonitors" class="text-xs text-gray-500">
              正在加载显示器信息...
            </div>
            <div v-if="monitorError" class="text-xs text-red-400">
              {{ monitorError }}
            </div>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">输出配置</h3>
          <div class="space-y-3">
            <label class="text-xs text-gray-300 block">Obsidian Vaults</label>
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
              尚未配置 Vault，请添加
            </div>
            <!-- Add vault form -->
            <div class="flex gap-2">
              <input v-model="newVaultName" type="text" placeholder="名称"
                class="w-1/3 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
              <input v-model="newVaultPath" type="text" placeholder="路径 (如 /Users/.../Obsidian Vault)"
                class="flex-1 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
              <button @click="addVault" :disabled="!newVaultName.trim() || !newVaultPath.trim()"
                class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-30 rounded-lg text-xs text-primary transition-colors shrink-0">
                添加
              </button>
            </div>
          </div>
        </div>

        <div>
          <label class="text-xs text-gray-300 block mb-2">Logseq Graphs</label>
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
            尚未配置 Graph，请添加
          </div>
          <!-- Add graph form -->
          <div class="flex gap-2">
            <input v-model="newGraphName" type="text" placeholder="名称"
              class="w-1/3 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
            <input v-model="newGraphPath" type="text" placeholder="路径 (如 /Users/.../Logseq/graph)"
              class="flex-1 bg-darker border border-gray-700 rounded-lg px-2 py-1.5 text-xs text-gray-100 placeholder:text-gray-500 focus:border-primary focus:outline-none" />
            <button @click="addGraph" :disabled="!newGraphName.trim() || !newGraphPath.trim()"
              class="px-3 py-1.5 bg-primary/20 hover:bg-primary/30 disabled:opacity-30 rounded-lg text-xs text-primary transition-colors shrink-0">
              添加
            </button>
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">快捷键</h3>
          <div class="bg-darker rounded-lg px-3 py-2 text-sm text-gray-400 border border-gray-700">
            闪念胶囊: Alt + Space
          </div>
        </div>

        <div>
          <h3 class="text-sm font-medium text-gray-300 mb-3">调试工具</h3>
          <div class="space-y-3">
            <button
              @click="exportLogs"
              :disabled="isExportingLogs"
              class="w-full px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:opacity-50 rounded-lg text-sm text-gray-200 transition-colors flex items-center justify-center gap-2"
            >
              {{ isExportingLogs ? '导出中…' : '📤 导出日志' }}
            </button>
            <span v-if="exportError" class="text-xs text-red-400 block">{{ exportError }}</span>
          </div>
        </div>
      </div>

      <!-- Default Prompt Modal -->
      <div v-if="showDefaultPromptModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showDefaultPromptModal = false">
        <div class="bg-dark rounded-2xl w-[500px] max-h-[80vh] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">默认分析 Prompt</h3>
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
              关闭
            </button>
          </div>
        </div>
      </div>

      <!-- Default Summary Prompt Modal -->
      <div v-if="showDefaultSummaryPromptModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showDefaultSummaryPromptModal = false">
        <div class="bg-dark rounded-2xl w-[500px] max-h-[80vh] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">默认日报 Prompt</h3>
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
              关闭
            </button>
          </div>
        </div>
      </div>

      <!-- AI-004: Default Tag Categories Modal -->
      <div v-if="showDefaultTagCategoriesModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showDefaultTagCategoriesModal = false">
        <div class="bg-dark rounded-2xl w-[400px] max-h-[80vh] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">默认标签分类</h3>
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
              关闭
            </button>
          </div>
        </div>
      </div>

      <!-- Template Library Modal -->
      <div v-if="showTemplateLibraryModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click.self="showTemplateLibraryModal = false">
        <div class="bg-dark rounded-2xl w-[500px] max-h-[80vh] overflow-hidden border border-gray-700">
          <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h3 class="text-lg font-semibold">模板库</h3>
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
              关闭
            </button>
          </div>
        </div>
      </div>

      <div class="px-6 py-4 border-t border-gray-700 flex items-center justify-between gap-3">
        <div class="flex flex-col">
          <span v-if="saveStatus === 'ok'" class="text-green-400 text-xs flex items-center gap-1">
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20"><path d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"/></svg>
            已保存
          </span>
          <span v-else-if="saveStatus === 'err'" class="text-red-400 text-xs flex items-center gap-1">
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/></svg>
            保存失败
          </span>
          <span v-if="saveError" class="text-xs text-red-400 mt-1">{{ saveError }}</span>
          <span v-else-if="!saveStatus" class="text-xs text-transparent select-none">placeholder</span>
        </div>
        <div class="flex gap-3">
          <button
            @click="$emit('close')"
            class="px-4 py-2 rounded-lg text-sm text-gray-300 hover:bg-gray-700 hover:text-white transition-colors"
          >
            取消
          </button>
          <button
            @click="saveSettings"
            :disabled="isSaving"
            class="px-4 py-2 bg-primary rounded-lg text-sm font-medium text-white hover:bg-blue-600 disabled:opacity-50 transition-colors"
          >
            {{ isSaving ? '保存中…' : '保存' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import { writeFile, writeTextFile, readTextFile } from '@tauri-apps/plugin-fs'
import { showError, showSuccess } from '../stores/toast.js'

const emit = defineEmits(['close'])

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

// API Connection test state
const isTestingConnection = ref(false)
const connectionTestResult = ref(null)

// Model info state
const isLoadingModelInfo = ref(false)
const analysisModelInfo = ref(null)
const summaryModelInfo = ref(null)

// Window whitelist/blacklist tag management
const whitelistTags = ref([])
const blacklistTags = ref([])
const newWhitelistTag = ref('')
const newBlacklistTag = ref('')

// AI-004: Tag categories
const tagCategoriesText = ref('')
const showDefaultTagCategoriesModal = ref(false)
const defaultTagCategoriesContent = ref([])

// Preset templates for summary prompt
const presetTemplates = [
  {
    id: 'default',
    name: '默认模板',
    description: '结构化日报，包含时间线、关键成果和问题',
    content: null // Will be loaded from backend
  },
  {
    id: 'concise',
    name: '简洁模板',
    description: '简洁摘要，仅列出主要工作项',
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
    name: '详细模板',
    description: '详细日报，包含时间分析和工作建议',
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
  logseq_graphs: '[]'
})

// SMART-003: Work time status for learning progress display
const workTimeStatus = ref(null)

// SMART-004: Monitor settings
const monitors = ref([])
const isLoadingMonitors = ref(false)
const monitorError = ref('')
const isScreenshotEnabled = ref(true) // Will be set based on backend capability

// DATA-006: Multi Obsidian Vault support
const vaults = ref([])
const newVaultName = ref('')
const newVaultPath = ref('')

// INT-002: Logseq graph support
const graphs = ref([])
const newGraphName = ref('')
const newGraphPath = ref('')

const loadSettings = async () => {
  try {
    const loaded = await invoke('get_settings')
    settings.value = { ...settings.value, ...loaded }
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
    workTimeStatus.value = await invoke('get_work_time_status')
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
    monitors.value = await invoke('get_monitors')
  } catch (err) {
    console.error('Failed to load monitors:', err)
    // If get_monitors is not available, screenshot feature is likely disabled
    if (String(err).includes('not found') || String(err).includes('not registered')) {
      isScreenshotEnabled.value = false
    } else {
      monitorError.value = `加载显示器信息失败: ${err}`
    }
  } finally {
    isLoadingMonitors.value = false
  }
}

// SMART-003: Format work time periods for display
const formatWorkTimePeriods = (periods) => {
  if (!periods || periods.length === 0) {
    return '未检测到'
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

const removeWhitelistTag = (index) => {
  whitelistTags.value.splice(index, 1)
}

const addBlacklistTag = () => {
  const tag = newBlacklistTag.value.trim()
  if (tag && !blacklistTags.value.includes(tag)) {
    blacklistTags.value.push(tag)
    newBlacklistTag.value = ''
  }
}

const removeBlacklistTag = (index) => {
  blacklistTags.value.splice(index, 1)
}

const validateSettings = () => {
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
    return '截图间隔必须在 1-60 分钟之间'
  }

  // Validate change threshold
  if (settings.value.change_threshold < 1 || settings.value.change_threshold > 20) {
    return '变化阈值必须在 1-20% 之间'
  }

  // Validate max silent minutes
  if (settings.value.max_silent_minutes < 5 || settings.value.max_silent_minutes > 120) {
    return '最大静默时间必须在 5-120 分钟之间'
  }

  return null
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
    showSuccess('设置已保存')
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

const removeVault = (index) => {
  const wasDefault = vaults.value[index].is_default
  vaults.value.splice(index, 1)
  if (wasDefault && vaults.value.length > 0) {
    vaults.value[0].is_default = true
  }
}

const setDefaultVault = (index) => {
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

const removeGraph = (index) => {
  const wasDefault = graphs.value[index].is_default
  graphs.value.splice(index, 1)
  if (wasDefault && graphs.value.length > 0) {
    graphs.value[0].is_default = true
  }
}

const setDefaultGraph = (index) => {
  graphs.value.forEach((g, i) => { g.is_default = i === index })
}

const exportLogs = async () => {
  isExportingLogs.value = true
  exportError.value = ''

  try {
    // Get log content
    const logContent = await invoke('get_logs_for_export')

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
      showSuccess('日志导出成功')
    }
  } catch (err) {
    console.error('Failed to export logs:', err)
    if (err !== 'Log file does not exist') {
      exportError.value = `导出失败: ${err}`
      showError(err)
    } else {
      exportError.value = '日志文件不存在'
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
  showSuccess('已重置为默认 Prompt，保存后生效')
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
  showSuccess('已重置为默认标签分类，保存后生效')
}

// API Connection test
const testConnection = async () => {
  if (!settings.value.api_base_url || !settings.value.model_name) {
    showError('请先填写 API Base URL 和分析模型')
    return
  }

  // For non-Ollama endpoints, API Key is required
  const isOllama = isOllamaEndpoint(settings.value.api_base_url)
  if (!isOllama && !settings.value.api_key) {
    showError('请先填写 API Key')
    return
  }

  isTestingConnection.value = true
  connectionTestResult.value = null

  try {
    const result = await invoke('test_api_connection_with_ollama', {
      apiBaseUrl: settings.value.api_base_url,
      apiKey: settings.value.api_key || null,
      modelName: settings.value.model_name
    })
    connectionTestResult.value = result
    if (result.success) {
      showSuccess(`连接成功 (${result.latency_ms}ms)`)
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
const getModelInfo = async (type) => {
  const modelName = type === 'analysis' ? settings.value.model_name : settings.value.summary_model_name
  if (!modelName) {
    showError('请先填写模型名称')
    return
  }

  isLoadingModelInfo.value = true

  try {
    const result = await invoke('get_model_info', {
      apiBaseUrl: settings.value.api_base_url,
      apiKey: settings.value.api_key,
      modelName: modelName
    })

    if (type === 'analysis') {
      analysisModelInfo.value = result
    } else {
      summaryModelInfo.value = result
    }

    if (result.error) {
      showError(result.error)
    } else if (result.context_window) {
      showSuccess(`${modelName}: ${result.context_window / 1000}K tokens`)
    } else {
      showSuccess('模型信息不可用，请参考模型文档')
    }
  } catch (err) {
    console.error('Failed to get model info:', err)
    showError(err)
  } finally {
    isLoadingModelInfo.value = false
  }
}

// AI-005: Ollama support
const ollamaModels = ref([])
const isLoadingOllamaModels = ref(false)
const ollamaModelError = ref('')

// Check if the current endpoint is an Ollama endpoint
const isOllamaEndpoint = (url) => {
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
    showError('请先填写 Base URL')
    return
  }

  isLoadingOllamaModels.value = true
  ollamaModelError.value = ''

  try {
    const result = await invoke('get_ollama_models', {
      baseUrl: settings.value.api_base_url
    })

    if (result.success) {
      ollamaModels.value = result.models
      if (result.models.length === 0) {
        ollamaModelError.value = '未找到已安装的模型，请使用 ollama pull <model> 安装'
      } else {
        showSuccess(`找到 ${result.models.length} 个模型`)
      }
    } else {
      ollamaModelError.value = result.message
      showError(result.message)
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
const selectOllamaModel = (modelName) => {
  settings.value.model_name = modelName
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
  showSuccess('已重置为默认 Prompt，保存后生效')
}

// Template Library functions
const showTemplateLibrary = async () => {
  // Load default template content from backend
  try {
    const defaultPrompt = await invoke('get_default_summary_prompt')
    presetTemplates[0].content = defaultPrompt
  } catch (err) {
    console.error('Failed to get default summary prompt:', err)
  }
  showTemplateLibraryModal.value = true
}

const applyTemplate = (template) => {
  if (template.content) {
    settings.value.summary_prompt = template.content
    showTemplateLibraryModal.value = false
    showSuccess(`已应用模板: ${template.name}`)
  } else {
    showError('模板内容为空')
  }
}

// Import/Export functions
const exportTemplate = async () => {
  const currentPrompt = settings.value.summary_prompt || ''
  if (!currentPrompt.trim()) {
    showError('当前 Prompt 为空，无法导出')
    return
  }

  try {
    const templateData = {
      version: '1.0',
      name: '我的日报模板',
      description: '自定义日报模板',
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
      showSuccess('模板导出成功')
    }
  } catch (err) {
    console.error('Failed to export template:', err)
    showError(`导出失败: ${err}`)
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
      showError('无效的模板文件：缺少 content 字段')
      return
    }

    // Check for {records} placeholder
    if (!templateData.content.includes('{records}')) {
      showError('模板缺少 {records} 占位符，请确保模板包含此占位符')
      return
    }

    settings.value.summary_prompt = templateData.content
    showSuccess(`导入成功: ${templateData.name || '未命名模板'}`)
  } catch (err) {
    console.error('Failed to import template:', err)
    if (err instanceof SyntaxError) {
      showError('导入失败: JSON 格式无效')
    } else {
      showError(`导入失败: ${err}`)
    }
  }
}

onMounted(() => {
  loadSettings()
})
</script>
