# Story 1.3: 日报生成模板优化

Status: ready-for-dev

## Story

作为一个 DailyLogger 用户，
我希望能够自定义日报的生成格式，包括标题格式和记录来源选择，
以便生成的日报更符合我的工作习惯和 Obsidian 笔记规范。

## Acceptance Criteria

### AC1 - 自定义日报标题格式
- Given 用户在设置界面配置日报标题格式
- When 用户输入自定义格式（如"工作日志 - {date}"或"Daily Report - {date}"）
- Then 生成的日报文件名和标题使用该格式，其中 `{date}` 被替换为实际日期

### AC2 - 选择包含/排除手动记录
- Given 用户在设置界面勾选/取消勾选"包含闪念胶囊记录"
- When 生成日报时
- Then 根据设置决定是否将 source_type='manual' 的记录纳入日报生成

### AC3 - Markdown 格式符合 Obsidian 规范
- Given 日报生成完成
- When 用户在 Obsidian 中打开日报
- Then Markdown 格式正确渲染，包括标题层级、列表格式、代码块（如有）

### AC4 - 设置界面支持
- Given 用户打开设置界面
- When 查看日报生成配置区域
- Then 显示标题格式输入框和"包含闪念胶囊"复选框，并有清晰的说明文字

## Tasks / Subtasks

- [ ] Task 1: 扩展数据库 Schema (AC: 1, 2)
  - [ ] 在 settings 表添加 `summary_title_format` 字段（默认值："工作日报 - {date}"）
  - [ ] 在 settings 表添加 `include_manual_records` 字段（默认值：1/true）
  - [ ] 在 Settings 结构体添加对应字段
  - [ ] 添加数据库迁移脚本

- [ ] Task 2: 修改 Rust 后端 synthesis 模块 (AC: 1, 2, 3)
  - [ ] 修改 `generate_daily_summary()` 支持标题格式替换
  - [ ] 修改记录获取逻辑，根据 `include_manual_records` 设置过滤记录
  - [ ] 确保 Markdown 输出符合 Obsidian 规范（标题使用 #，列表使用 -，时间格式规范）
  - [ ] 更新 DEFAULT_SUMMARY_PROMPT 模板，确保输出格式标准化

- [ ] Task 3: 修改前端设置界面 (AC: 4)
  - [ ] 在 SettingsModal.vue 日报生成区域添加标题格式输入框
  - [ ] 添加"包含闪念胶囊"复选框
  - [ ] 添加字段说明文字（placeholder/hint）
  - [ ] 确保设置保存时包含新字段

- [ ] Task 4: 编写测试 (All ACs)
  - [ ] Rust 单元测试：标题格式替换逻辑
  - [ ] Rust 单元测试：记录过滤逻辑（include_manual_records=true/false）
  - [ ] Rust 单元测试：Markdown 输出格式验证
  - [ ] 前端组件测试：新设置字段显示和保存

## Dev Notes

### 技术需求

1. **修改 Rust 后端** - synthesis/mod.rs 和 memory_storage/mod.rs
2. **修改 Vue 前端** - SettingsModal.vue
3. **数据库迁移** - ALTER TABLE 添加新列
4. **TailwindCSS only** - 前端样式使用 Tailwind
5. **测试必须通过** - `cargo test` 和 `npm run test`

### 架构合规要求

- 遵循现有模块结构（synthesis 负责生成，memory_storage 负责数据）
- 使用 params![] 宏进行参数化查询
- 使用 Mutex 保护全局数据库连接
- 前端使用 Vue 3 Composition API 和 `<script setup>` 语法

### 现有实现分析

**synthesis/mod.rs 关键代码路径：**
```
generate_daily_summary()
  → get_settings_sync() 获取配置
  → get_all_today_records_for_summary() 获取记录
  → 使用 summary_prompt 或 DEFAULT_SUMMARY_PROMPT
  → 替换 {records} 占位符
  → 调用 OpenAI API
  → 写入 Obsidian 路径/YYYY-MM-DD.md
```

**需要修改的位置：**
1. `memory_storage/mod.rs`: Settings 结构体添加 `summary_title_format` 和 `include_manual_records`
2. `memory_storage/mod.rs`: init_database() 添加 ALTER TABLE 迁移
3. `memory_storage/mod.rs`: get_settings_sync() 和 save_settings_sync() 处理新字段
4. `synthesis/mod.rs`: 根据设置过滤记录
5. `synthesis/mod.rs`: 应用标题格式到输出文件

### 文件结构要求

**后端文件：**
- `src-tauri/src/memory_storage/mod.rs` - 数据存储和 Settings 结构体
- `src-tauri/src/synthesis/mod.rs` - 日报生成逻辑

**前端文件：**
- `src/components/SettingsModal.vue` - 设置界面

**测试文件：**
- `src-tauri/src/synthesis/mod.rs` - #[cfg(test)] 模块
- `src/components/__tests__/SettingsModal.spec.js` - 前端测试

### 测试要求

**Rust 测试重点：**
1. 标题格式替换：`"{date}"` → `"2026-03-14"`
2. 记录过滤：`include_manual_records=false` 时排除 source_type='manual'
3. Markdown 格式：验证输出包含正确的标题层级

**前端测试重点：**
1. 新设置字段正确显示
2. 复选框状态切换正常
3. 保存设置包含新字段

### Obsidian Markdown 规范要点

1. 标题层级：一级标题 `#` 用于文档标题，二级 `##` 用于章节
2. 列表格式：使用 `- ` 作为无序列表
3. 代码块：使用 ``` 包裹，支持语法高亮
4. 链接：支持 `[[]]` 内部链接语法
5. 标签：支持 `#tag` 格式

## Previous Story Intelligence

### 从 CORE-001 学习的经验

1. **测试模式**：每个 AC 对应多个测试用例确保验收通过
2. **文件修改模式**：直接修改现有组件，遵循现有代码风格
3. **Tailwind 类名**：使用 `text-xs text-gray-300` 作为 label 样式
4. **保存反馈**：成功后 800ms 自动关闭，显示绿色勾号

### 从 CORE-002 学习的经验

1. **视图状态管理**：使用 ref() 管理组件状态
2. **复用组件**：优先复用现有组件而非创建新组件
3. **hover 状态**：所有可交互元素添加 hover 效果

## Project Structure Notes

### 现有项目结构

```
src-tauri/src/
├── lib.rs                     # 应用入口
├── main.rs                    # Tauri 主进程
├── memory_storage/
│   └── mod.rs                 # 数据存储（本次修改）
├── synthesis/
│   └── mod.rs                 # 日报生成（本次修改）
└── ...

src/
├── App.vue                    # 主界面容器
├── components/
│   ├── SettingsModal.vue      # 设置模态框（本次修改）
│   └── ...
```

### Settings 结构体现有字段

```rust
pub struct Settings {
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub model_name: Option<String>,
    pub screenshot_interval: Option<i32>,
    pub summary_time: Option<String>,
    pub obsidian_path: Option<String>,
    pub auto_capture_enabled: Option<bool>,
    pub last_summary_path: Option<String>,
    pub summary_model_name: Option<String>,
    pub analysis_prompt: Option<String>,
    pub summary_prompt: Option<String>,
    pub change_threshold: Option<i32>,
    pub max_silent_minutes: Option<i32>,
    // 新增字段：
    // pub summary_title_format: Option<String>,
    // pub include_manual_records: Option<bool>,
}
```

## References

- [Source: architecture.md#2.2 后端模块] - synthesis/mod.rs 职责描述
- [Source: architecture.md#5.2 settings 表] - 数据库 Schema 定义
- [Source: PRD.md#6.3 AI 日报生成] - 原始产品需求
- [Source: epics.md#Epic 1] - 所属 Epic 信息
- [Source: src-tauri/src/synthesis/mod.rs] - 现有日报生成实现
- [Source: src-tauri/src/memory_storage/mod.rs] - 现有数据存储实现
- [Source: src/components/SettingsModal.vue] - 现有设置界面

## Dev Agent Record

### Agent Model Used

BMAD create-story Workflow

### Implementation Summary

待 dev-story 执行后填写

### Tests Added

待 dev-story 执行后填写

### File List

待 dev-story 执行后填写

### Change Log

- Story 创建完成 (Date: 2026-03-14)
- 状态：ready-for-dev