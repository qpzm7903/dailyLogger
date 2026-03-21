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
- **团队协作** — 用户注册、团队管理、记录共享

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

## 项目结构

```
src/                        # Vue 3 前端
  ├── App.vue               # 主界面
  └── components/           # UI 组件
      ├── SettingsModal.vue
      ├── QuickNoteModal.vue
      ├── ScreenshotModal.vue
      ├── ScreenshotGallery.vue
      ├── DailySummaryViewer.vue
      ├── HistoryViewer.vue
      ├── TagCloud.vue
      ├── ReportComparisonModal.vue
      ├── LogViewer.vue
      ├── TimelineVisualization.vue
      ├── PluginPanel.vue
      ├── BackupModal.vue
      ├── LoginModal.vue
      └── TeamPanel.vue

src-tauri/src/              # Rust 后端
  ├── main.rs               # 应用入口、托盘、日志初始化
  ├── lib.rs                # AppState、模块导出
  ├── auto_perception/      # 定时截图 + AI 分析
  ├── manual_entry/         # 速记 + 文件读取
  ├── memory_storage/       # SQLite CRUD + 全文搜索
  ├── synthesis/            # AI 日报/周报/月报生成
  ├── offline_queue/        # 离线任务队列
  ├── backup/               # 数据备份恢复
  ├── crypto/               # API Key 加密存储
  ├── hardware/             # 硬件抽象层（跨平台支持）
  ├── auth/                 # 用户认证（注册/登录/会话）
  ├── team/                 # 团队协作（团队管理/记录共享）
  ├── notion.rs             # Notion API 集成
  ├── slack.rs              # Slack Webhook 集成
  ├── github.rs             # GitHub API 集成
  ├── ollama.rs             # Ollama 本地模型管理
  ├── fine_tuning.rs        # 模型微调
  ├── plugin.rs             # 插件系统
  └── timeline.rs           # 时间线数据生成
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

**最新版本**: v1.107.0
- 维护检查：CI 全部通过，测试覆盖完善
- 无开放 Issues，依赖最新，Clippy 无警告
- 前端测试 939 个，Rust 测试 435 个

v1.106.0 更新:
- 维护检查：CI 全部通过，测试覆盖完善
- 无开放 Issues，依赖最新，Clippy 无警告

v1.104.0 更新:
- 维护检查：CI 全部通过，测试覆盖完善
- 无开放 Issues，依赖最新，Clippy 无警告

v1.102.0 更新:
- 维护检查：CI 全部通过，测试覆盖完善
- 无开放 Issues，依赖最新，Clippy 无警告

v1.100.0 更新:
- 新增：仅截图模式 - 启动后只记录截图，延迟 AI 分析 (#65)
- 新增：截图支持用户手动补充备注 (#66)
- 修复：Release 发布时包含版本变更内容 (#67)

v1.99.0 更新:
- 新增：按日期重新分析记录功能 (#64)
- 创建 ReanalyzeByDateModal 组件，支持选择任意日期重新分析

v1.98.0 更新:
- 维护检查：CI 全部通过，测试覆盖完善
- 无开放 Issues，依赖最新，Clippy 无警告

v1.94.0 更新:
- 新增：批量重新分析当天记录功能 (#63)
- AI 分析失败时仍保存截图并加入离线队列

v1.85.0 更新:
- 修复 4 个用户反馈问题 (#59, #60, #61, #62)
- 模型无法连接时截图自动保存并加入离线队列
- 历史记录展示内容改为多行显示
- 提高深色主题文字对比度

v1.58.0 更新:
- UI 重构：Sidebar + Header + Dashboard 架构，现代化深色玻璃态界面
- 修复：启动分析后没有截图出现 (#57)
- 修复：Windows 上闪念胶囊快捷键无效 (#58)

**早期版本 (v1.0.0 ~ v1.57.0)**:
- v1.32.0: 改进错误提示和用户反馈（i18n 支持）
- v1.31.0: 前端代码全量 TypeScript 迁移
- v1.30.0: 引入 TypeScript 基础设施
- v1.29.0: 更新 vitest 到 v4，jsdom 到 v29
- v1.28.0: 迁移 vue-i18n 从 v9 到 v11
- v1.26.0: 修复本地地址 API 请求被代理拦截问题 (#47)
- v1.24.0~v1.25.0: 依赖更新、Windows 启动问题修复
- v1.20.0~v1.23.0: 团队协作、本地 AI 微调、Ollama 管理
- v1.0.0 ~ v1.19.0: 核心功能实现（自动捕获、手动记录、AI 日报、多平台支持等）

## 贡献

欢迎提交 Issue 和 Pull Request！

## License

MIT
