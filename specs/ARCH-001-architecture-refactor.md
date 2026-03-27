# ARCH-001 架构收口与可维护性重构

**目标版本**: v3.4.0 ~ v3.6.0
**优先级**: HIGH
**类型**: 渐进式架构重构（保持功能兼容）

## 背景

当前仓库已经具备较完整的功能集合，但核心边界开始影响后续迭代效率：

- `src/App.vue` 当前约 654 行，同时承担布局编排、状态管理、定时器、快捷键、Tauri 事件监听和业务动作触发。
- `src-tauri/src/main.rs` 当前约 749 行，同时承担日志初始化、托盘、命令注册、平台分支和启动流程。
- `src-tauri/src/lib.rs` 当前约 524 行，同时混杂公共函数、HTTP/Proxy 逻辑、路径逻辑和全局状态。
- 前端组件直接使用 `invoke(...)`，调用边界分散，业务动作难以复用和测试。
- Settings / Record / Session 等契约前后端分散维护，字段继续增长时容易产生漂移。

这类问题已经不是“代码风格”问题，而是后续功能继续增长时会持续抬高改动成本和回归风险。

## 重新思考后的结论

本轮不采用“一次性 Clean Architecture 重写”。

更适合当前仓库的方案是：先收口入口和调用边界，再抽离服务层，最后统一契约和状态。也就是说，先解决最痛的耦合点，而不是先追求一套看起来完整但落地成本高的大架构。

## 本轮目标

- 让前端根组件只负责应用壳和页面编排，不再直接承接大量业务副作用
- 让前端组件不再直接散落 Tauri IPC 调用
- 让 Tauri 启动入口只负责装配，不再堆业务细节
- 让命令层和业务实现之间出现明确边界
- 为后续 Vault、标签、导出等功能恢复稳定迭代速度

## 不在范围内

- 不进行 UI 视觉重设计
- 不在 v3.4.0 引入 Pinia、Vue Router、事件总线或代码生成链路
- 不做数据库 schema 重写或破坏性迁移
- 不重做现有核心功能交互
- 不引入“插件化”或“微前端”级别的复杂抽象

## 当前痛点与目标映射

| 当前问题 | 直接影响 | 本轮解法 |
|---|---|---|
| `App.vue` 过重 | 改一个功能容易波及根组件和多个 modal 流程 | 提取 `app/` 层和 feature actions |
| 组件直接 `invoke(...)` | 调用分散、难测、难替换 | 建立统一 `shared/api/tauri` client |
| `main.rs` 过重 | 启动逻辑、托盘逻辑、命令注册耦合 | 提取 `bootstrap/` 模块 |
| 命令与业务混杂 | 难形成清晰 service 边界 | 建立 `commands -> services -> infrastructure` 分层 |
| 契约重复维护 | Settings/Session 字段演进风险高 | 在三期统一契约和错误模型 |

## 目标目录形态

### 前端目标形态

```text
src/
├── app/
│   ├── AppShell.vue
│   ├── AppModals.vue
│   └── useAppBootstrap.ts
├── features/
│   ├── capture/
│   │   ├── actions.ts
│   │   └── types.ts
│   ├── dashboard/
│   │   └── actions.ts
│   ├── reports/
│   │   └── actions.ts
│   └── sessions/
│       └── actions.ts
├── shared/
│   ├── api/
│   │   └── tauri/
│   │       ├── client.ts
│   │       └── commands.ts
│   ├── types/
│   └── ui/
└── components/
    └── ...
```

### Rust 后端目标形态

```text
src-tauri/src/
├── bootstrap/
│   ├── app_builder.rs
│   ├── commands.rs
│   ├── logging.rs
│   └── tray.rs
├── commands/
│   ├── capture.rs
│   ├── reports.rs
│   ├── sessions.rs
│   └── settings.rs
├── services/
│   ├── capture_service.rs
│   ├── report_service.rs
│   ├── session_service.rs
│   └── settings_service.rs
├── infrastructure/
│   ├── http.rs
│   ├── paths.rs
│   └── state.rs
└── existing domain modules...
```

说明：

- 这不是要求一次性迁移所有文件。
- 现有 `memory_storage/`、`synthesis/`、`session_manager/` 等模块仍可保留，只是逐步让外部通过更清晰的入口访问。

## 分阶段实施方案

### Phase 1: v3.4.0 架构收口一期

**目标**: 先拆入口和调用边界。

**前端交付物**:

- 从 `src/App.vue` 提取 `AppShell.vue`、`AppModals.vue`、`useAppBootstrap.ts`
- 把截图、报告、设置、时段等业务动作抽到 feature actions
- 新增 `src/shared/api/tauri/client.ts`，统一封装 `invoke(...)`
- 新增命令常量或函数封装，避免组件层硬编码命令字符串

**后端交付物**:

- 从 `src-tauri/src/main.rs` 提取 `bootstrap/logging.rs`
- 提取 `bootstrap/tray.rs`
- 提取 `bootstrap/commands.rs` 或类似模块，集中组织 `generate_handler![]`
- `main.rs` 只保留 builder 装配和 setup 调用

**阶段验收**:

- `src/components/`、`src/components/layout/` 中不再直接出现 `invoke(...)`
- `src/App.vue` 不再同时承接定时器、事件监听、业务动作和 modal 装配
- `src-tauri/src/main.rs` 缩减为启动装配文件
- `npm run test`、`npm run typecheck` 至少保持现状可通过

### Phase 2: v3.5.0 架构收口二期

**目标**: 抽出 service 边界，降低命令层和具体模块实现的耦合。

**交付物**:

- 新建 `commands/` 模块，所有 `#[tauri::command]` 统一收口
- 新建 `services/` 模块，命令层只负责参数转换、权限/上下文装配和错误映射
- 将 Settings、Session、Report、Capture 四个高频领域优先迁出 service 边界
- 前端按功能目录整理，减少根组件和 `components/` 横向耦合

**阶段验收**:

- 命令层不再直接承载复杂业务编排
- 新增功能时，前端修改范围可控制在单一 feature 内
- Settings / Report / Session 至少 3 个领域具备可识别的 service 入口

### Phase 3: v3.6.0 架构收口三期

**目标**: 统一契约、错误模型和全局状态边界。

**交付物**:

- 梳理 Settings / Session / Report / Record 前后端字段映射
- 统一错误返回模型和前端错误处理入口
- 收敛新增全局状态的方式，减少继续扩散的 `Lazy<Mutex<...>>`
- 建立架构约束文档，明确“组件不能直接 IPC、命令不能直接堆业务”的规则

**阶段验收**:

- 高频契约拥有明确的单一来源或同步规范
- 新增字段时不再需要多处手工比对后猜测兼容性
- 后续功能版本可以在稳定边界上继续推进，而不是反复回到入口文件打补丁

## 文件级重构映射

| 当前文件 | 主要问题 | 目标落点 |
|---|---|---|
| `src/App.vue` | 根组件承担过多副作用和业务动作 | `src/app/AppShell.vue` + `src/app/AppModals.vue` + `src/app/useAppBootstrap.ts` |
| `src/components/layout/Dashboard.vue` 相关交互 | 页面动作分散在根组件 | `src/features/dashboard/actions.ts` |
| `src/types/tauri.ts` | 类型集中但与后端演进容易漂移 | 三期统一契约规则，必要时按领域拆分 |
| `src-tauri/src/main.rs` | 启动入口过重 | `src-tauri/src/bootstrap/*` |
| `src-tauri/src/lib.rs` | 公共能力与状态耦合 | `src-tauri/src/infrastructure/*` + 更清晰的公共导出 |
| `src-tauri/src/memory_storage/mod.rs` | 既是存储模块又暴露大量命令和模型 | 二期开始通过 `services/` 和 `commands/` 收口 |

## 验收条件（Given/When/Then）

### AC1 - 行为兼容

- Given 用户按当前方式使用截图、速记、日报、周报、设置、时段分析
- When 完成本轮分阶段重构
- Then 用户可见功能行为与当前版本保持一致

### AC2 - 前端调用边界统一

- Given 任意 Vue 组件或 composable
- When 需要调用 Tauri 命令
- Then 通过统一的 IPC client 或 feature actions 完成，而不是直接散落 `invoke(...)`

### AC3 - Tauri 启动入口瘦身

- Given `src-tauri/src/main.rs`
- When 完成一期重构
- Then 文件只保留启动装配职责，日志、托盘、命令注册不再全部内联

### AC4 - 命令层与业务层解耦

- Given 任意 `#[tauri::command]`
- When 命令触发业务逻辑
- Then 命令函数只做入口转换和调用 service，而不堆积复杂业务编排

### AC5 - 回归验证不退化

- Given 重构完成
- When 执行类型检查、前端测试和关键流程验证
- Then 结果不低于重构前基线

## 实施顺序

1. 先建立 `ARCH-001` 文档和版本计划，冻结重构边界
2. 从前端开始，拆 `App.vue` 和 IPC client
3. 再拆 `main.rs`，把日志、托盘、命令装配移走
4. 建立 `commands -> services` 调用边界
5. 补回归测试和验证清单
6. 最后再做契约统一和全局状态收敛

## 风险与缓解

| 风险 | 说明 | 缓解措施 |
|---|---|---|
| 根组件拆分引发状态同步问题 | modal、定时器、事件监听容易在迁移时遗漏 | 先提取 bootstrap 和 actions，再移动 UI 容器 |
| Tauri 命令迁移导致命令名或返回结构变化 | 前端已有调用较多 | 一期先包一层 client，不改命令名；二期再逐步收口 |
| 托盘与快捷键逻辑回归 | 平台相关代码易在拆分时出问题 | 将 tray/shortcut 逻辑独立模块化，并保留现有行为测试清单 |
| 契约统一过早导致大面积联动 | Settings/Session 字段多，改动面大 | 契约统一延后到三期，先做盘点和入口收口 |

## 回滚策略

- 每一期都保持小步提交和可发布状态
- 若某一阶段出现不稳定，可仅回滚该阶段的壳层或命令层调整，不影响既有业务模块
- 在契约统一前，不做破坏性字段重命名或数据库层重构
