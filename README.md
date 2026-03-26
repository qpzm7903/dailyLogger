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

### 生产构建

```bash
npm run tauri build
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

**最新版本**: v3.0.0
- 工作时段感知分析：捕获与分析解耦、时段批量上下文分析、用户编辑功能
- 手动触发分析：支持批量时段选择和分析
- 日报生成适配：按时段组织，用户摘要优先
- GitHub 集成移除（CLEAN-001）

**近期版本 (v2.10.0 ~ v2.15.0)**:
- v2.15.0: UX-4 Dashboard 重组、Header 状态栏、记录列表分页
- v2.14.0: UX-3 Sidebar 升级、UX-2 按钮规范化
- v2.13.0: SESSION-004 手动触发分析功能
- v2.12.0: Epic-8 SESSION-003 完成后版本发布
- v2.11.0: (same as v2.10.0)
- v2.10.0: 今日摘要 Widget：实时统计记录数、时间跨度、活跃时段
- v2.9.0: 截图重新分析按钮
- v2.8.0: 截图质量过滤
- v2.6.0: Slack/钉钉通知集成
- v2.5.0: GitHub 工时统计展示
- v2.4.0: GitHub API 集成验证
- v2.3.0: Logseq 导出支持
- v2.2.0: Notion 导出完善
- v2.1.0: 自定义 API Headers
- v2.0.0: 架构瘦身，移除 fine_tuning/plugin 模块

**早期版本 (v1.0.0 ~ v1.100.0)**:
- v1.100.0: 仅截图模式、截图备注
- v1.99.0: 按日期重新分析
- v1.94.0: 批量重新分析
- v1.85.0: 问题修复 (#59-62)
- v1.58.0: UI 重构、Sidebar 架构
- v1.46.0: Tailwind CSS v4
- v1.45.0: SettingsModal 拆分
- v1.30-1.32: TypeScript 迁移
- v1.0-1.19: 核心功能实现

## 贡献

欢迎提交 Issue 和 Pull Request！

## License

MIT
