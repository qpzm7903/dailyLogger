# Story 11.1: DATA-007 - 多语言日报导出

Status: ready-for-dev

## Story

As a DailyLogger user,
I want to export my daily report in multiple languages,
so that I can share my work progress with international team members or maintain records in my preferred language.

## Acceptance Criteria

1. **Given** 用户有当日记录，**When** 用户选择目标语言并点击"生成多语言日报"，**Then** 系统生成对应语言的 Markdown 日报
2. **Given** 用户已选择默认语言，**When** 生成日报，**Then** 自动使用用户偏好的语言
3. **Given** 支持的语言列表，**When** 用户打开语言选择，**Then** 显示所有支持的语言选项（包括：中文、English、日语、英语等）
4. **Given** 日报生成成功，**When** 用户查看，**Then** 显示文件路径并提供打开选项
5. **Given** 日报生成失败，**When** 错误发生，**Then** 显示具体错误信息并允许重试
6. **Given** 用户未配置语言偏好，**When** 首次使用，**Then** 默认使用系统语言或中文

## Tasks / Subtasks

- [ ] Task 1: 数据库扩展 - 多语言配置字段 (AC: #3, #6)
  - [ ] 1.1 在 Settings 表添加 `preferred_language` 字段 (TEXT, 默认 "zh-CN")
  - [ ] 1.2 在 Settings 表添加 `supported_languages` 字段 (TEXT, JSON 数组)
  - [ ] 1.3 更新 Settings struct 和相关 CRUD 函数
  - [ ] 1.4 添加数据库迁移逻辑

- [ ] Task 2: Rust 后端 - 多语言日报生成核心逻辑 (AC: #1, #2, #4, #5)
  - [ ] 2.1 在 synthesis/mod.rs 扩展 `generate_daily_summary()` 支持语言参数
  - [ ] 2.2 创建 `translate_report()` 函数调用 AI 翻译
  - [ ] 2.3 创建 `get_supported_languages()` 函数返回支持的语言列表
  - [ ] 2.4 在 main.rs 的 `generate_handler![]` 中注册新/扩展命令
  - [ ] 2.5 编写单元测试
    - 测试语言切换
    - 测试翻译质量
    - 测试默认语言 fallback

- [ ] Task 3: 前端 - 多语言日报 UI (AC: #1, #2, #3, #4, #5, #6)
  - [ ] 3.1 在 App.vue 添加语言选择器下拉菜单
  - [ ] 3.2 修改"生成日报"按钮逻辑支持语言参数
  - [ ] 3.3 添加语言偏好保存功能
  - [ ] 3.4 添加多语言日报生成 loading 状态和成功/错误提示
  - [ ] 3.5 显示生成的语言版本日报路径

- [ ] Task 4: 端到端测试 (AC: All)
  - [ ] 4.1 前端测试: 语言选择和日报生成交互
  - [ ] 4.2 Rust 集成测试: 完整多语言日报生成流程

## Dev Notes

### Architecture Context

**关键架构决策**:
- 复用现有 `synthesis/mod.rs` 的日报生成模式
- 翻译使用现有 AI API，无需额外服务
- 多语言日报文件名格式: `{原文件名}.{语言代码}.md` (如 `工作日报-2026-03-26.en.md`)
- 支持语言独立存储，也可覆盖同一文件（用户可选）

**必须遵循的代码模式** [Source: architecture.md]:
- Tauri Command: `#[command]` + async
- 错误处理: `Result<T, String>` + `.map_err(|e| e.to_string())`
- 数据库访问: 使用全局 `DB_CONNECTION` Mutex
- 时区处理: 使用 `and_local_timezone(chrono::Local)` 避免 UTC 偏移问题

### Key Existing Code to Reuse

**synthesis/mod.rs** - 复用以下函数:
- `generate_daily_summary()` - 现有日报生成逻辑
- `format_records_for_summary()` - 格式化记录为 AI prompt 文本
- `filter_records_by_settings()` - 根据设置过滤记录
- LLM 调用模式 (reqwest client + JSON request)

**memory_storage/mod.rs** - 参考以下函数:
- `get_today_records_sync()` - 时间范围查询模式
- `get_settings_sync()` / `save_settings_sync()` - Settings CRUD

### Supported Languages

```rust
/// 支持的语言列表
const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("zh-CN", "中文"),
    ("en", "English"),
    ("ja", "日本語"),
    ("ko", "한국어"),
    ("es", "Español"),
    ("fr", "Français"),
    ("de", "Deutsch"),
];

/// 语言代码到文件后缀的映射
fn get_language_suffix(lang: &str) -> &str {
    match lang {
        "zh-CN" => "",
        "en" => ".en",
        "ja" => ".ja",
        "ko" => ".ko",
        "es" => ".es",
        "fr" => ".fr",
        "de" => ".de",
        _ => "",
    }
}
```

### Translation Prompt

```rust
const TRANSLATION_PROMPT: &str = r#"你是一个专业的技术文档翻译助手。请将以下 Markdown 格式的工作日报翻译成{language}。

要求：
1. 保持 Markdown 格式不变
2. 技术术语保持准确
3. 保持原意的专业性
4. 输出纯翻译结果，不要有其他说明文字

原文：
{original_report}

请翻译："#;
```

### Database Migration

在 `init_database()` 中添加:
```rust
let _ = conn.execute(
    "ALTER TABLE settings ADD COLUMN preferred_language TEXT DEFAULT 'zh-CN'",
    [],
);
let _ = conn.execute(
    "ALTER TABLE settings ADD COLUMN supported_languages TEXT DEFAULT '[\"zh-CN\",\"en\",\"ja\"]'",
    [],
);
```

### File Naming Convention

多语言日报文件名格式: `{原文件名}.{语言代码}.md`
- 例如: `工作日报-2026-03-26.en.md` (英语版)
- 例如: `工作日报-2026-03-26.ja.md` (日语版)

### Project Structure Notes

**需要修改的文件**:
- `src-tauri/src/memory_storage/mod.rs` - 添加语言配置字段
- `src-tauri/src/synthesis/mod.rs` - 扩展日报生成支持多语言
- `src-tauri/src/main.rs` - 注册/扩展命令
- `src/App.vue` - 添加语言选择器
- `src/components/SettingsModal.vue` - 添加语言偏好设置

**前端组件参考**: 复用现有下拉选择器模式

### Testing Requirements

**必须测试的场景**:
1. 语言切换: 从中文切换到英文，日报内容正确翻译
2. 默认语言: 未配置语言时使用系统默认或中文
3. 翻译质量: Markdown 格式保持正确
4. 空记录: 无记录时正确处理
5. 错误处理: AI 翻译失败时显示错误信息

**测试模式** (参考现有测试):
```rust
#[test]
fn generates_report_in_different_languages() {
    setup_test_db();
    // 测试生成英文日报
    // 测试生成日语日报
    // 验证格式保持 Markdown
}

#[test]
fn uses_default_language_when_not_set() {
    setup_test_db();
    // 验证默认语言 fallback
}
```

### Previous Story Intelligence

从 REPORT-001/REPORT-002 学到的经验:
- 时间边界计算使用 `and_local_timezone()` 而非 `.and_utc()`
- Settings 表使用 ALTER TABLE 添加新字段时忽略错误（兼容已有列）
- 复用 `format_records_for_summary()` 减少代码重复
- 文件命名使用清晰的后缀格式便于识别

### References

- [Source: architecture.md#2.2] - 后端模块架构
- [Source: architecture.md#3.3] - 日报生成流程（多语言扩展此流程）
- [Source: architecture.md#4.3] - 时区处理正确方式
- [Source: PRD.md#6.3] - 日报生成功能需求
- [Source: epics.md#Epic 11] - 数据增强与稳定性 Epic
- [Source: REPORT-001 story] - 周报生成实现模式（复用模式）
- [Source: REPORT-002 story] - 月报生成实现模式（复用模式）

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
