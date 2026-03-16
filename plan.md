# DailyLogger 项目规划

> 最后更新: 2026-03-16
> 当前版本: v1.9.0
> 下一版本: v1.10.0

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

---

## 当前版本: v1.10.0（CI/CD 完善与基础设施改进）

**目标**: 完善 CI/CD 发布流水线，补齐 Linux 构建，规范发布产物命名，改进日志系统。

**版本类型**: MINOR（新增 Linux 平台支持）

### 需求清单

| ID | 需求 | 优先级 | 来源 | 状态 |
|----|------|--------|------|------|
| INFRA-001 | 发布流水线补齐 Linux x64 构建 | High | prompt.md 构建矩阵要求 | 待开发 |
| INFRA-002 | 规范发布产物命名格式 | High | prompt.md 文件命名规范 | 待开发 |
| INFRA-003 | Release 自动发布（当前仅 draft） | Medium | 发布流程完善 | 待开发 |
| INFRA-004 | 日志文件轮转（当前 Rotation::NEVER） | Medium | 生产可靠性 | 待开发 |

### 详细说明

#### INFRA-001: 补齐 Linux x64 构建

当前 build.yml 的 Release 构建仅覆盖 macOS arm64 和 Windows x64，缺少 Linux x64。prompt.md 明确要求三平台构建矩阵：
- Windows x64 (windows-latest) ✅ 已有
- macOS arm64 (macos-latest) ✅ 已有
- Linux x64 (ubuntu-latest) ❌ **缺失**

需要添加 `build-linux` job，安装 Tauri 系统依赖后构建 `.tar.gz` 产物。

#### INFRA-002: 规范发布产物命名

prompt.md 要求的命名规范：
- Windows: `DailyLogger-vX.Y.Z-windows-x64.exe` （当前是 `DailyLogger-vX.Y.Z-windows-portable.zip`）
- Linux: `DailyLogger-vX.Y.Z-linux-x64.tar.gz`
- macOS: `DailyLogger-vX.Y.Z-macos-arm64.dmg` （当前由 tauri-action 自动命名）

#### INFRA-003: Release 自动发布

当前 create-release job 创建 draft release，但没有后续步骤将其发布。需要在所有平台构建完成后自动 publish release。

#### INFRA-004: 日志文件轮转

当前 `setup_logging()` 使用 `Rotation::NEVER`，日志文件会无限增长。改为按日轮转 (`Rotation::DAILY`)，保留最近 7 天日志。

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

**P3 远期功能**:
- 时间线可视化（图形化展示一天工作流）

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
