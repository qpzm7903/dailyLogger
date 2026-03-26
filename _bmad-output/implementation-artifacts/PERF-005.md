# Story 10.5: 多语言支持 (i18n)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a DailyLogger user,
I want to use the app in my preferred language,
so that I can understand and use all features effectively regardless of my language background.

**来源**: plan.md 未来规划 - 多语言支持

## Background

### Epic 10 定位

```
Epic 10: 体验极致化
├── PERF-001: AI 配置完善（代理支持） ✅ 已完成
├── PERF-002: 新用户引导 ✅ 已完成
├── PERF-003: 性能优化 - 截图加载 ✅ 已完成
├── PERF-004: 性能优化 - 数据库查询 ✅ 已完成
├── PERF-005: 多语言支持 (i18n) ← 当前
└── PERF-006: 浅色主题支持
```

### 当前 i18n 基础设施

项目已有完整的 vue-i18n 基础设施：

| 文件 | 说明 |
|------|------|
| `src/i18n.ts` | vue-i18n 配置，包含 `setLocale()` / `getLocale()` helpers |
| `src/locales/en.json` | 英文翻译（完整） |
| `src/locales/zh-CN.json` | 中文翻译（完整） |
| `src/components/settings/BasicSettings.vue` | 语言选择 UI（第 381-405 行） |

**当前 `detectLanguage()` 逻辑** (`src/i18n.ts`):
1. 检查 `localStorage.getItem('dailylogger-locale')`
2. 降级到 `navigator.language` 检测浏览器语言
3. 默认返回 `'en'`

**现有语言选择 UI** (`BasicSettings.vue`):
- 两个按钮：`English` 和 `简体中文`
- 点击调用 `changeLanguage(lang)` → `setLocale(lang)` → 保存到 localStorage

### 待完成项

当前实现使用 **localStorage** 存储语言偏好，存在以下问题：

1. **不持久化到后端** — 语言设置未保存到 `settings` 表，重启应用后可能被浏览器自动语言检测覆盖
2. **首次启动流程不完整** — 虽然 `detectLanguage()` 能检测浏览器语言，但用户设置的语言偏好应该优先

## Acceptance Criteria

1. **语言切换**
   - Given 用户在设置中选择语言
   - When 切换到 English
   - Then 所有界面文字显示英文，且设置持久化到后端

2. **中文支持**
   - Given 用户在设置中选择语言
   - When 切换到 中文
   - Then 所有界面文字显示中文，且设置持久化到后端

3. **首次启动自动检测**
   - Given 用户首次启动应用
   - When 自动检测系统语言
   - Then 默认使用检测到的语言（如果支持 en 或 zh-CN）

4. **日报不受影响**
   - Given 应用已生成日报
   - When 用户切换语言
   - Then 已生成的日报内容不受影响（仅界面变化）

## Tasks / Subtasks

- [ ] Task 1: 后端设置持久化 (AC: #1, #2, #3)
  - [ ] Subtask 1.1: 在 settings 表添加 `language` 字段（如果不存在）
  - [ ] Subtask 1.2: 后端 `get_settings` / `save_settings` 支持 language 字段
  - [ ] Subtask 1.3: 前端 SettingsModal 同步 language 字段

- [ ] Task 2: 语言设置初始化逻辑 (AC: #3)
  - [ ] Subtask 2.1: 应用启动时从后端加载语言设置
  - [ ] Subtask 2.2: 如果后端无设置，使用 `navigator.language` 检测并保存
  - [ ] Subtask 2.3: 确保 localStorage 优先级与后端一致

- [ ] Task 3: 语言切换流程 (AC: #1, #2)
  - [ ] Subtask 3.1: `changeLanguage()` 调用 Tauri command 保存到后端
  - [ ] Subtask 3.2: 保存成功后更新 vue-i18n locale
  - [ ] Subtask 3.3: 保持 localStorage 作为 fallback

- [ ] Task 4: 回归测试 (AC: #4)
  - [ ] Subtask 4.1: 验证日报生成不受语言切换影响
  - [ ] Subtask 4.2: 运行 `npm test` 确保无回归

## Dev Notes

### 关键架构约束

1. **技术栈**: Vue 3 + vue-i18n + Tauri v2
2. **持久化方案**: 使用后端 SQLite `settings` 表存储语言偏好
3. **优先级**: 后端设置 > localStorage > 浏览器语言检测
4. **不引入新依赖**: 使用现有的 vue-i18n

### 文件树组件（需修改）

```
src/
├── i18n.ts                      # 修改 detectLanguage 逻辑，优先使用后端设置
├── App.vue                      # 应用启动时加载语言设置
└── components/
    └── settings/
        └── BasicSettings.vue     # 修改 changeLanguage，调用后端保存

src-tauri/src/
├── memory_storage/
│   ├── mod.rs                  # 如果需要，添加 language 字段处理
│   └── schema.rs               # 如果需要，添加 language 列
└── main.rs                     # 注册可能的 Tauri commands
```

### 实现方案

**后端设置流程**:
```typescript
// src/i18n.ts
async function loadLanguageFromBackend(): Promise<Locale> {
  try {
    const settings = await invoke<Settings>('get_settings')
    if (settings.language === 'en' || settings.language === 'zh-CN') {
      return settings.language
    }
  } catch (e) {
    console.warn('Failed to load language from backend:', e)
  }
  return null
}

function detectLanguage(): Locale {
  // 1. 检查 localStorage
  const stored = localStorage.getItem('dailylogger-locale')
  if (stored && (stored === 'en' || stored === 'zh-CN')) {
    return stored as Locale
  }

  // 2. 降级到浏览器语言
  const browserLang = navigator.language || (navigator as { userLanguage?: string }).userLanguage
  if (browserLang && browserLang.startsWith('zh')) {
    return 'zh-CN'
  }
  return 'en'
}
```

**语言切换流程**:
```typescript
// BasicSettings.vue
async function changeLanguage(lang: Locale) {
  setLocale(lang) // 更新 vue-i18n 和 localStorage
  // 保存到后端
  const settings = { ...localSettings.value, language: lang }
  await invoke('save_settings', { settings })
}
```

### 数据库字段（如果需要添加）

```sql
-- 如果 settings 表没有 language 列，需要添加
ALTER TABLE settings ADD COLUMN language TEXT DEFAULT 'en';
```

### 注意事项

1. **向后兼容**: 如果后端返回的 settings 没有 language 字段，使用 localStorage 或浏览器检测
2. **日报内容**: 日报内容存储在文件系统中，与语言设置无关
3. **测试验证**: 确保 `npm test` 通过后再提交

### References

- [Source: src/i18n.ts] - vue-i18n 配置和 helpers
- [Source: src/locales/en.json] - 英文翻译
- [Source: src/locales/zh-CN.json] - 中文翻译
- [Source: src/components/settings/BasicSettings.vue#381-405] - 语言选择 UI
- [Source: _bmad-output/implementation-artifacts/PERF-001.md] - PERF-001 参考（后端设置持久化模式）

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
