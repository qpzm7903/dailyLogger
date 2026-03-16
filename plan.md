# DailyLogger 项目规划

> 最后更新: 2026-03-16
> 当前版本: v1.10.0
> 下一版本: v1.11.0

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

---

## 中期规划: v1.11.0（功能扩展）

**目标**: 完成剩余 backlog 功能，提升产品完整度。

| ID | 需求 | 故事点 | 状态 |
|----|------|--------|------|
| DATA-006 | 多 Obsidian Vault 支持 | 3pts | Backlog |
| REPORT-004 | 报告对比分析 | 5pts | Backlog |

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
| DEBT-001 | 数据库 Schema 版本化迁移（settings 表已 25+ 字段） | CORE/DATA/AI 回顾 | HIGH | 待开发 |
| DEBT-002 | 离线队列 ScreenshotAnalysis 重试为空操作 | 代码审查 | MEDIUM | 待开发 |
| DEBT-003 | 统一测试数据库 Schema 初始化 | AI 回顾 | MEDIUM | 待开发 |
| DEBT-004 | 前端组件测试覆盖率 | 多 Epic 回顾 | MEDIUM | 待开发 |
| DEBT-005 | 学习数据持久化（SilentPatternTracker / WorkTimePatternLearner） | SMART 回顾 | MEDIUM | 待开发 |
| DEBT-006 | 硬件抽象层（窗口/显示器/截图 API） | SMART 回顾 | LOW | 待开发 |

---

## 已知问题

| 问题 | 影响 | 关联版本 |
|------|------|----------|
| 历史 GitHub Release (v1.0.0~v1.9.0) 均为 Draft 未发布 | 用户无法下载旧版本 | v1.0.0 ~ v1.9.0 |

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
