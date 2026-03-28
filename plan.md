# DailyLogger 项目规划

> 最后更新: 2026-03-28
> 当前版本: v4.0.0（进行中）
> 下一版本: v4.1.0（规划中）
> 当前 Milestone: v4.0.0 组件颜色 CSS 变量化

---

## 当前 Milestone：v4.0.0 组件颜色 CSS 变量化

**目标**: 完成浅色主题系统，将所有硬编码颜色迁移到 CSS 变量

**版本策略**:

| 版本 | 类型 | 目标 |
|------|------|------|
| v4.0.0 | MAJOR | 组件颜色全面迁移到 CSS 变量，完成浅色主题系统 |

**已完成事项**:

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| DEBT-003 | 组件颜色迁移到 CSS 变量 | 8 | P0 | ✅ 已完成 |

**DEBT-003 完成范围**:
- 迁移 21 个 Vue 组件文件
- 迁移 13 个测试文件
- 新增 CSS 变量: `--color-border`, `--color-border-subtle`, `--color-text-muted`
- 替换模式: `bg-darker` → `bg-[var(--color-surface-0)]` 等
- 所有 964 个测试通过

**待办事项**:

| ID | 需求 | 故事点 | 优先级 | 状态 |
|----|------|--------|--------|------|
| - | 手动验证浅色主题 UI 效果 | 2 | P1 | 待办 |

---

## 未来 Milestone 概要

| 版本 | 方向 | 说明 |
|------|------|------|
| v4.0.0 | MAJOR | 组件颜色 CSS 变量化（进行中） |
| v4.1.0 | MINOR | 待定 |

---

## 最近 10 个已完成版本摘要

### v4.0.0 — 组件颜色 CSS 变量化 ✅ (进行中)
- 完成 DEBT-003：21 个 Vue 组件迁移到 CSS 变量
- 新增 CSS 变量：`--color-border`, `--color-border-subtle`, `--color-text-muted`
- 所有组件支持浅色主题切换
- 964 前端测试全部通过

### v3.10.0 — 数据库迁移系统完善 ✅
- 完成 DEBT-006/007/008：集成 run_migrations() 到 init_database()，移除冗余 ALTER TABLE 语句
- 建立 schema_version 和 schema_migrations 表追踪数据库版本
- 实现幂等迁移执行器，支持结构化版本回滚
- 508 Rust 测试 + 964 前端测试全部通过

### v3.9.0 — 多 Vault 自动选择 ✅
- 基于窗口标题自动选择输出 Vault
- 前端添加窗口标题匹配模式配置 UI
- 新增 12 个 vault 相关单元测试

### v3.8.0 — 多维度输出增强 ✅
- 新增自定义导出模板功能，支持用户自定义 Markdown 导出格式
- 实现模板占位符：`{{date}}`, `{{time}}`, `{{content}}`, `{{source_type}}`, `{{source_icon}}`, `{{tags}}`
- 新增 `get_default_export_template` / `get_default_record_entry_template` 后端命令
- 更新 ExportModal UI，支持自定义模板编辑器和预览

### v3.7.1 — 标签管理增强 ✅
- 标签颜色后端化：后端存储标签颜色，前端从缓存获取
- 实现 `get_tag_colors()` / `set_tag_color()` 命令
- 三级回退逻辑：缓存 → 默认颜色表 → 哈希分配

### v3.6.0 — 架构收口三期 ✅
- 统一前后端契约：修复 Settings 和 LogRecord 类型定义
- 建立结构化错误模型：AppError 枚举和统一错误处理
- 收敛全局状态：建立 infrastructure/state.rs 文档规范
- 建立架构约束文档：specs/ARCH-010-architecture-constraints.md

### v3.5.0 — 架构收口二期 ✅
- 抽取 Settings/Session/Report/Capture 四个领域 service 边界
- 命令层重构为薄 IPC 适配器，业务逻辑下沉到 services
- 补齐回归基线：486 Rust 测试 + 964 前端测试全部通过

### v3.4.0 — 架构收口一期 ✅
- 提取前端应用壳：AppShell、AppModals、useAppBootstrap
- 建立统一 Tauri IPC Client 和 feature actions，组件不再直接散落 `invoke()`
- 拆分 `main.rs`：提取 bootstrap/logging.rs、bootstrap/tray.rs、bootstrap/commands.rs

### v3.3.0 — 体验极致化续 ✅
- 新用户引导、截图加载优化、数据库查询优化、多语言支持、浅色主题全部落地
- Epic 10 完成，整体体验和性能明显提升

### v3.2.0 — AI 代理配置 ✅
- AI API 请求支持 HTTP 代理和认证
- 补充测试连接模型和前端折叠配置面板

### v3.1.1 — CI 修复 ✅
- 修复 Build and Release workflow 中 release 发布链路异常
- 保证版本发布流程恢复可用
