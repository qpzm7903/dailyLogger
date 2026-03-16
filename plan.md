# DailyLogger 项目规划

> 最后更新: 2026-03-16
> 当前版本: v1.11.0 ✅ 已发布
> 下一版本: v1.12.0

---

## 已完成版本

### v1.0.0 ~ v1.9.0（Sprint 1 完成）

Sprint 1 完成了 5 大 Epic（87 故事点，24 个 Story），覆盖核心功能、智能捕获、AI 能力、数据管理和报告功能。

**已交付功能汇总**:

| Epic | 故事数 | 状态 |
|------|--------|------|
| Epic 1: 核心功能完善 (CORE-001~008) | 8 | 全部完成 |
| Epic 2: 智能捕获优化 (SMART-001~004) | 4 | 全部完成 |
| Epic 3: AI 能力提升 (AI-001~005) | 5 | 全部完成 |
| Epic 4: 数据管理与检索 (DATA-001~005) | 5 | 全部完成 |
| Epic 5: 周报月报功能 (REPORT-001~003) | 3 | 全部完成 |

**关键技术成果**:
- Tauri v2 跨平台桌面应用 (macOS/Windows/Linux)
- 自动截屏 + OpenAI Vision AI 分析
- AES-256-GCM API Key 加密存储
- 离线模式 + 任务队列 + 指数退避重试
- Ollama 本地模型集成
- 智能静默时段 + 工作时间自动识别
- 全文搜索、标签系统、数据导出、备份恢复
- 周报/月报/自定义周期报告
- 日志系统 (tracing + 文件输出)
- 397 个 Rust 测试 + 16 个前端测试
- 对比分析报告（REPORT-004）、多 Obsidian Vault 支持（DATA-006）

### v1.10.0（CI/CD 完善与基础设施改进）✅ 已发布

**目标**: 完善 CI/CD 发布流水线，补齐 Linux 构建，规范发布产物命名，改进日志系统。

| ID | 需求 | 状态 |
|----|------|------|
| INFRA-001 | 发布流水线补齐 Linux x64 构建 | ✅ 完成 |
| INFRA-002 | 规范发布产物命名格式 | ✅ 完成 |
| INFRA-003 | Release 自动发布（不再仅 draft） | ✅ 完成 |
| INFRA-004 | 日志文件按日轮转（保留 7 天） | ✅ 完成 |
| 额外 | CI 测试矩阵补齐 Linux runner | ✅ 完成 |
| 额外 | 修复 silent_tracker 全局测试隔离 | ✅ 完成 |
| 额外 | 修复 Linux 构建：升级 ubuntu-24.04 + 补齐 libgbm/libegl 依赖 | ✅ 完成 |

---

## 当前迭代: v1.11.0（功能扩展与 Bug 修复）✅ 已发布

**目标**: 新增报告对比分析、多 Obsidian Vault 输出支持，修复月报路径覆盖 Bug。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| REPORT-004 | 报告对比分析 | 5pts | ✅ 完成 |
| DATA-006 | 多 Obsidian Vault 支持 | 3pts | ✅ 完成 |
| FIX-001 | 月报路径覆盖日报路径 Bug | 1pt | ✅ 完成 |

### REPORT-004: 报告对比分析

**功能**: 选择两个时间段，AI 生成对比分析报告（工作量变化、重点转移、效率趋势等）。

**后端变更** (`src-tauri/src/`):
- `synthesis/mod.rs`: 新增 `compare_reports()` Tauri 命令，查询两个时段的记录并发送给 AI 对比分析
- `memory_storage/mod.rs`: Settings 新增 `comparison_report_prompt` 字段（可自定义对比 prompt）
- `main.rs`: 注册新命令到 `generate_handler![]`

**前端变更** (`src/`):
- 新建 `components/ReportComparisonModal.vue`: 双时段选择器 + 生成按钮
- `App.vue`: 新增对比报告入口按钮与状态管理
- 对比结果复用 `DailySummaryViewer.vue` 展示

**验收条件**:
- 用户可选择两个任意时段进行对比
- AI 返回结构化对比报告（Markdown 格式）
- 报告保存到 Obsidian 输出目录

### DATA-006: 多 Obsidian Vault 支持

**功能**: 支持配置多个 Obsidian Vault 输出路径，可设置默认 Vault。

**后端变更** (`src-tauri/src/`):
- `memory_storage/mod.rs`: Settings 新增 `obsidian_vaults` 字段（JSON 数组: `[{name, path, is_default}]`），保留 `obsidian_path` 作为回退兼容
- `synthesis/mod.rs`: 4 个报告生成函数（daily/weekly/monthly/custom）支持指定 Vault 或使用默认 Vault
- `manual_entry/mod.rs`: `open_obsidian_folder` 打开默认 Vault

**前端变更** (`src/`):
- `components/SettingsModal.vue`: 替换单一路径输入为 Vault 列表管理（添加/删除/设默认）
- `App.vue`: 报告生成时支持选择目标 Vault（或使用默认）

**验收条件**:
- 支持添加、删除、编辑多个 Vault
- 可设置一个默认 Vault
- 所有报告类型均支持多 Vault
- 旧版单路径配置自动迁移

### FIX-001: 月报路径覆盖日报路径 Bug

**问题**: `generate_monthly_report()` 将输出路径保存到 `last_summary_path`（与日报共享），导致生成月报后日报路径引用被覆盖。

**修复**:
- `memory_storage/mod.rs`: 新增 `last_monthly_report_path` 字段
- `synthesis/mod.rs`: 月报输出路径保存到 `last_monthly_report_path`
- `App.vue`: 加载设置时读取 `last_monthly_report_path`
- 回归测试：生成月报后验证日报路径未被修改

---

## 长期规划: v2.0.0+（集成与扩展）

**目标**: 与第三方工具集成，扩展应用场景。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| INT-001 | Notion 导出支持 | 5pts | Backlog |
| INT-002 | Logseq 导出支持 | 3pts | Backlog |
| INT-003 | GitHub 工时统计 | 8pts | Backlog |
| INT-004 | Slack/钉钉通知 | 5pts | Backlog |

**远期功能**:
- 时间线可视化（图形化展示一天工作流）
- 多语言支持 (i18n)
- 插件系统架构
- 移动端适配（Tauri Mobile）
- 团队协作模式
- 本地 AI 模型深度集成（模型管理、微调）

---

## 技术债务追踪

| ID | 描述 | 来源 | 优先级 | 状态 |
|----|------|------|--------|------|
| DEBT-001 | 数据库 Schema 版本化迁移（settings 表已 33 字段，ALTER TABLE 链已 21 条） | CORE/DATA/AI 回顾 | HIGH | 待开发 |
| DEBT-002 | 离线队列 ScreenshotAnalysis 重试为空操作 | 代码审查 | MEDIUM | 待开发 |
| DEBT-003 | 统一测试数据库 Schema 初始化 | AI 回顾 | MEDIUM | 待开发 |
| DEBT-004 | 前端组件测试覆盖率 | 多 Epic 回顾 | MEDIUM | 待开发 |
| DEBT-005 | 学习数据持久化（SilentPatternTracker / WorkTimePatternLearner） | SMART 回顾 | MEDIUM | 待开发 |
| DEBT-006 | 硬件抽象层（窗口/显示器/截图 API） | SMART 回顾 | LOW | 待开发 |

---

## 已知问题

| 问题 | 影响 | 关联版本 | 状态 |
|------|------|----------|------|
| ~~历史 GitHub Release (v1.0.0~v1.9.0) 均为 Draft 未发布~~ | ~~用户无法下载旧版本~~ | v1.0.0 ~ v1.9.0 | ✅ 已修复（v1.4.0~v1.9.0 已发布，重复 Draft 已清理） |
| 月报生成覆盖日报路径（`last_summary_path`） | 生成月报后日报路径丢失 | v1.8.0+ | ✅ 已修复 (v1.11.0 FIX-001) |
| Windows/macOS 构建产物无法打开 (#15) | 用户下载后无法启动应用 | v1.10.0 | ✅ 已修复 (v1.11.0 NSIS 安装程序) |

---

## 版本发布检查清单

每次发布新版本时，需确认以下事项：

1. [ ] 所有需求开发完成，测试通过
2. [ ] `cargo fmt && cargo clippy -- -D warnings && cargo test` 全绿
3. [ ] `npm run test` 全绿
4. [ ] 更新版本号：`package.json` / `Cargo.toml` / `tauri.conf.json`
5. [ ] 提交版本升级 commit
6. [ ] 创建并推送 tag: `git tag vX.Y.Z && git push && git push --tags`
7. [ ] GitHub Release 构建成功，产物已发布
8. [ ] 关闭相关 Issues
9. [ ] 更新 README.md（如有必要）
10. [ ] 更新本文件状态
