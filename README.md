# DailyLogger

AI 驱动的工作流记忆与日报生成桌面应用。自动截屏分析你的工作上下文，结合手动速记，在一天结束时生成结构化日报并输出到 Obsidian。

## 功能

### 核心功能
- **自动感知** — 定时截取屏幕，调用 OpenAI Vision API 分析当前工作内容
- **闪念胶囊** — 全局快捷键 `Alt+Space` 随时记录想法，不打断工作流
- **AI 日报生成** — 汇总全天记录，生成 Markdown 日报并保存到 Obsidian vault
- **截图回顾** — 浏览当日所有截图，点击查看大图
- **系统托盘** — 最小化到托盘，后台静默运行

### 高级功能
- **周报/月报** — 生成周报、月报和自定义时段报告
- **报告对比分析** — 对比两个时段的工作内容，分析变化趋势
- **标签系统** — 手动为记录添加标签，支持标签分类和颜色管理
- **全文搜索** — 基于 SQLite FTS5 的高性能全文搜索
- **多 Obsidian Vault** — 支持配置多个输出目录，可设置默认 Vault
- **智能静默检测** — 自动识别工作时间，智能调整截图间隔
- **窗口过滤** — 白名单/黑名单模式，只捕获关注的应用窗口
- **离线模式** — 网络断开时自动排队，恢复后重试
- **数据导出/备份** — 支持导出 JSON 和数据库备份恢复
### 集成功能
- **Obsidian 导出** — 报告自动写入 Obsidian Vault（支持多 Vault）
- **Logseq 导出** — 报告输出到 Logseq 图谱（支持多图谱）
- **Notion 导出** — 通过 API 写入 Notion 数据库
- **GitHub 工时统计** — 自动分析 GitHub 提交和 PR 活动计算工作时长
- **Slack 通知** — 将报告发送到 Slack 频道

### 其他功能
- **时间线可视化** — 图形化展示一天工作流程
- **多语言支持** — 支持中文/英文界面切换
- **Ollama 模型管理** — 直接在应用内拉取/删除本地 AI 模型

## 技术栈

| 层 | 技术 |
|---|---|
| 框架 | Tauri v2 |
| 前端 | Vue 3 + TailwindCSS |
| 后端 | Rust |
| 数据库 | SQLite (rusqlite) |
| AI | OpenAI Vision API (兼容 API) |

## 快速开始

### 环境要求

- Node.js ≥ 18
- Rust ≥ 1.70
- [Tauri v2 系统依赖](https://v2.tauri.app/start/prerequisites/)

### 安装与运行

```bash
# 克隆仓库
git clone https://github.com/qpzm7903/dailylogger.git
cd dailylogger

# 安装前端依赖
npm install

# 开发模式（带热重载）
npm run tauri dev
```

### 构建

#### 本地快速构建

```bash
npm run build:desktop:local
# 输出: src-tauri/target/local/
# 用途: 本地验证前端产物 + Rust 桌面二进制，不生成安装包
```

#### 正式发布构建

```bash
npm run build:desktop:release
# 输出: src-tauri/target/release/bundle/
```

## 配置

首次启动后点击右上角 ⚙️ 进入设置：

### 基础配置
| 配置项 | 说明 |
|--------|------|
| API Base URL | OpenAI 兼容 API 地址（支持 Ollama 本地模型） |
| API Key | API 密钥（Ollama 可留空） |
| Model | 模型名称（默认 `gpt-4o`，Ollama 使用如 `llama3.2-vision`） |
| 截图间隔 | 自动截屏间隔（分钟） |
| Obsidian Vaults | 日报输出目录（支持多个 Vault） |

### 高级配置
- **智能静默检测** — 自动识别工作时间，动态调整截图频率
- **窗口过滤** — 白名单/黑名单模式，只捕获关注的应用
- **多显示器支持** — 选择主显示器或所有显示器截图
- **自定义 Prompt** — 自定义 AI 分析和报告生成的提示词
- **标签分类** — 创建标签类别，组织管理标签

### 输出集成配置

#### Logseq 导出配置
1. 在设置中找到 **Logseq Graphs** 区域
2. 输入图谱名称（如 "Work"）和图谱根目录路径
3. 点击添加，首个图谱自动设为默认
4. 多个图谱时，点击 ☆ 设为默认
5. 报告将自动写入 `{graph-path}/pages/` 目录

#### Notion 导出配置
1. 在 Notion 中创建数据库，获取 Database ID
2. 创建 Integration 获取 API Key
3. 在设置中填入 API Key 和 Database ID
4. 点击测试连接验证配置

#### Obsidian 导出配置
1. 在设置中找到 **Obsidian Vaults** 区域
2. 添加一个或多个 Vault 路径
3. 设置默认 Vault
4. 报告将自动写入默认 Vault 根目录

#### GitHub 工时统计配置
1. 在 GitHub 设置中创建 Personal Access Token（需要 `repo` 权限）
2. 在设置中找到 **GitHub Work Time Statistics** 区域
3. 填入 GitHub Token（自动加密存储）
4. 添加监控仓库列表（格式：`owner/repo`，每行一个）
5. 点击测试连接验证配置
6. 日报生成时自动包含 GitHub 提交和 PR 活动统计

## 项目结构

```
src/                        # Vue 3 前端
  ├── App.vue               # 主界面
  └── components/           # UI 组件
      ├── layout/           # 布局组件
      │   ├── Sidebar.vue
      │   ├── Header.vue
      │   └── Dashboard.vue
      ├── settings/         # 设置子组件
      │   ├── BasicSettings.vue
      │   ├── AISettings.vue
      │   ├── CaptureSettings.vue
      │   └── OutputSettings.vue
      ├── SettingsModal.vue
      ├── QuickNoteModal.vue
      ├── QuickNoteWindow.vue
      ├── ScreenshotModal.vue
      ├── ScreenshotGallery.vue
      ├── DailySummaryViewer.vue
      ├── HistoryViewer.vue
      ├── TagCloud.vue
      ├── TagBadge.vue
      ├── TagFilter.vue
      ├── TagInput.vue
      ├── SearchPanel.vue
      ├── ReportDropdown.vue
      ├── ReportHistoryViewer.vue
      ├── ReportComparisonModal.vue
      ├── CustomReportModal.vue
      ├── ReanalyzeByDateModal.vue
      ├── TimelineVisualization.vue
      ├── BackupModal.vue
      ├── ExportModal.vue
      ├── OfflineBanner.vue
      ├── OfflineQueueModal.vue
      ├── LogViewer.vue
      └── Toast.vue

src-tauri/src/              # Rust 后端
  ├── main.rs               # 应用入口、托盘、日志初始化
  ├── lib.rs                # AppState、模块导出
  ├── auto_perception/      # 定时截图 + AI 分析
  ├── manual_entry/         # 速记 + 文件读取
  ├── memory_storage/       # SQLite CRUD + 全文搜索
  ├── synthesis/            # AI 日报/周报/月报生成
  ├── offline_queue.rs      # 离线任务队列
  ├── backup/               # 数据备份恢复
  ├── crypto/               # API Key 加密存储
  ├── hardware/             # 硬件抽象层（跨平台支持）
  ├── window_info/          # 窗口信息获取
  ├── export/               # 数据导出
  ├── notion.rs             # Notion API 集成
  ├── slack.rs              # Slack Webhook 集成
  ├── github.rs             # GitHub API 集成
  ├── ollama.rs             # Ollama 本地模型管理
  ├── timeline.rs           # 时间线数据生成
  ├── silent_tracker.rs     # 智能静默检测
  ├── work_time.rs          # 工作时间计算
  ├── performance.rs        # 性能监控
  ├── monitor.rs            # 多显示器支持
  └── network_status.rs     # 网络状态检测
```

## 开发

### 运行测试

```bash
# Rust 测试
cd src-tauri && cargo test

# 前端测试
npm run test
```

### 代码检查

```bash
cd src-tauri && cargo fmt && cargo clippy -- -D warnings
```

### Git hooks

```bash
# 安装 pre-commit hook（每次 clone 后执行一次）
git config core.hooksPath .githooks
```

## 平台支持

| 平台 | 截图方案 | 状态 | 下载 |
|------|---------|------|------|
| macOS (arm64) | xcap | ✅ | [.dmg](https://github.com/qpzm7903/dailyLogger/releases/latest) |
| Windows (x64) | Windows Graphics Capture API | ✅ | [安装版](https://github.com/qpzm7903/dailyLogger/releases/latest) / [免安装版](https://github.com/qpzm7903/dailyLogger/releases/latest) |
| Linux (x64) | xcap | ✅ | [.tar.gz](https://github.com/qpzm7903/dailyLogger/releases/latest) |

## 版本历史

查看 [Releases](https://github.com/qpzm7903/dailyLogger/releases) 获取完整更新日志。

**最新版本**: v4.2.0
- HistoryViewer 虚拟滚动：使用 @tanstack/vue-virtual 实现大数据量列表优化
- 配置阈值 100 条，超过后启用虚拟化渲染
- 修复虚拟化计数响应式问题，确保过滤后数据正确更新

**v4.1.1**:
- 前端组件测试覆盖率提升：完成 35+ 个 Vue 组件的测试覆盖
- 新增 40+ 测试文件，测试数量从约 800 增至 1165
- 62 个测试文件全部通过 (1165 tests)
- 覆盖所有主要 UI 组件：ErrorBoundary、Sidebar、Header、TodaySummaryWidget 等

**v4.0.0**:
- 组件颜色 CSS 变量化：完成浅色主题系统
- 迁移 21 个 Vue 组件到 CSS 变量
- 新增 CSS 变量：`--color-border`, `--color-border-subtle`, `--color-text-muted`
- 964 前端测试全部通过

**v3.10.0**:
- 数据库版本迁移机制：建立 schema_version 和 schema_migrations 表追踪
- 实现幂等迁移执行器，支持结构化版本回滚
- 添加 5 个迁移相关测试，验证版本追踪和幂等性

**v3.9.0（多 Vault 自动选择）**:
- 多 Vault 自动选择：基于窗口标题自动选择输出 Vault
- OutputSettings 添加"根据窗口标题自动选择 Vault"开关
- 每个 Vault 支持配置窗口标题匹配模式（多个用逗号分隔）
- Rust 后端 12 个 vault 相关测试全部通过

**v3.8.0（多维度输出增强）**:
- 多维度输出增强：自定义导出模板功能
- 支持模板占位符：`{{date}}`, `{{time}}`, `{{content}}`, `{{source_type}}`, `{{source_icon}}`, `{{tags}}`
- 新增 `get_default_export_template` / `get_default_record_entry_template` 后端命令
- ExportModal UI 支持自定义模板编辑器和预览

**v3.7.1（标签管理增强）**:
- 标签颜色后端化：后端存储标签颜色，前端从缓存获取
- 后端 `get_tag_colors()` / `set_tag_color()` 命令
- 三级回退逻辑：缓存 → 默认颜色表 → 哈希分配

**v3.6.0（架构收口三期）**:
- 统一前后端契约：修复 Settings 和 LogRecord 类型定义
- 建立结构化错误模型：AppError 枚举和统一错误处理
- 收敛全局状态：建立 infrastructure/state.rs 文档规范

**近期版本 (v3.0.0 ~ v3.5.0)**:
- v3.5.0: 架构收口二期 - 抽取四个领域 service 边界
- v3.0.0: 工作时段感知分析 + GitHub 移除
- v2.10.0 ~ v2.15.0: Dashboard 重组、Sidebar 升级、手动触发分析等
- v1.0.0 ~ v1.100.0: 核心功能、UI 重构、TypeScript 迁移等

## 贡献

欢迎提交 Issue 和 Pull Request！

## License

MIT
