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

**最新版本**: v4.6.0
- 消除 session_manager/session_service ~560 行重复代码
- AppResult 迁移完成：全部业务逻辑函数使用结构化错误类型 (~120+ 函数)
- Tauri 命令层统一 `.map_err(|e| e.to_string())` IPC 边界模式
- lib.rs/vision_api/memory_storage 服务层全部迁移至 AppResult

**v4.5.0**:
- Rust 错误类型统一：全量迁移至 AppError/AppResult 结构化错误体系，~100+ 函数
- 命令层统一使用 `.map_err(|e| e.to_string())` 模式，消除冗余 format! 包装
- 512 Rust + 1226 前端测试全部通过

**v4.4.3**:
- 修复 legacy 数据库 sessions 表未创建导致启动报错 (issue #89)
- 修复 ReportDropdown 被 backdrop-blur stacking context 遮挡 (issue #90)
- 修复 Settings 页面输入框和按钮颜色异常 (issue #91)

**v4.4.2**:
- StatisticsPanel SVG viewBox 修复：使用 Vue 动态绑定替代 Mustache 模板语法，确保图表正确缩放

**v4.4.0**:
- 数据分析增强：新增生产力趋势分析，支持"本周 vs 上周"、"本月 vs 上月"对比视图 (ANALYTICS-001)
- SVG 折线图展示日趋势数据
- 高峰时段分布显示 top 5 busiest hours

**v4.3.7**:
- 启动速度优化：实现懒加载机制、缓冲诊断写入、延迟 Tray 和 Backup Scheduler 初始化 (PERF-007)
- 数据库迁移修复：修复旧数据库 sessions 表缺失列问题 (issue #85)

**v4.3.0** ~ **v4.3.5**: 启动速度优化、Windows 启动修复、多次 Migration 幂等性修复、AI Settings 模板导入/导出、i18n 国际化

**更早版本**:
- v4.2.1 ~ v4.0.0: HistoryViewer 虚拟滚动、Tailwind CSS v4 升级、数据库版本迁移机制等
- v3.x: 多 Vault 自动选择、自定义导出模板、标签颜色后端化等
- 详见 [Releases](https://github.com/qpzm7903/dailyLogger/releases) 获取完整更新日志

## 贡献

欢迎提交 Issue 和 Pull Request！

## License

MIT
