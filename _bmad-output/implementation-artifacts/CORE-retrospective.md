# Epic 复盘：核心功能完善 (CORE)

**复盘日期**: 2026-03-15
**Epic 状态**: 已完成 (6/6 stories)
**参与者**: Weiyicheng (Project Lead), Bob (Scrum Master), Alice (Product Owner), Charlie (Senior Dev), Dana (QA Engineer)

---

## 一、Epic 总览

| 指标 | 数值 |
|------|------|
| Story 总数 | 6 |
| 已完成 | 6 (100%) |
| 总 Story Points | 19 (3+5+3+3+2+3) |
| 前端测试数 | 从 0 增长至 ~92 |
| 后端测试数 | 从 ~20 增长至 ~157 |
| 代码审查通过率 | 100% |
| 新增 Tauri 命令 | 8+ (get_records_by_date_range, get_logs_for_export, get_log_file_path, get_auto_capture_status, tray_quick_note, open_obsidian_folder 等) |
| 新增前端组件 | 3 个 (Toast.vue, errors.js, toast.js) |
| 新增后端模块 | 1 个 (crypto/) |
| 数据库新增字段 | 2 个 (summary_title_format, include_manual_records) |

### Stories 清单

| ID | 标题 | Points | 状态 | 关键产出 |
|----|------|--------|------|----------|
| CORE-001 | 设置界面优化 | 3 | done | UI 一致性规范、32 个前端测试 |
| CORE-002 | 截图画廊增强 | 5 | done | 视图切换、日期筛选、分页加载、41+92 测试 |
| CORE-003 | 日报生成模板优化 | 3 | done | 自定义标题格式、手动记录过滤、21 个 Rust 测试 |
| CORE-004 | 错误处理与用户提示 | 3 | done | Toast 通知系统、错误分类、日志导出、87+39 测试 |
| CORE-005 | 系统托盘菜单完善 | 2 | done | 动态托盘菜单、快速记录、打开 Obsidian 文件夹 |
| CORE-006 | API Key 加密存储 | 3 | done | AES-256-GCM 加密、自动迁移、内存安全清除、157 总测试 |

---

## 二、做得好的地方 (What Went Well)

### 1. 渐进式 UI 规范建立

CORE-001 作为第一个 Story，建立了 TailwindCSS 设计语言规范（`text-xs text-gray-300` for labels、`hover:bg-gray-700` for buttons），后续所有 Story 都遵循了这个规范。这种「先定规范，再做功能」的模式有效避免了 UI 风格不一致问题。

### 2. 测试文化从零到成熟

- CORE-001 起步时前端测试几乎为零，通过 6 个 Story 逐步积累至 92+ 前端测试
- 每个 Story 都有明确的测试要求，AC 验收与测试一一对应
- 后端测试覆盖了核心路径：日期范围查询、加密解密、错误分类、日志导出
- 测试模式在 Story 间传承：CORE-001 建立的 CSS 类名测试模式被 CORE-002 继承

### 3. 组件复用策略

- CORE-002 复用 `ScreenshotModal.vue` 实现快速预览，避免创建冗余组件
- CORE-004 的 Toast 系统被后续所有 Story 的错误处理采用
- CORE-005 复用 `QuickNoteModal.vue` 的输入模式用于托盘快速记录
- 「优先复用现有组件而非创建新组件」成为团队共识

### 4. Previous Story Intelligence 机制

Story 文件中的「Previous Story Intelligence」章节非常有效。每个新 Story 都从前序 Story 提取了具体可操作的经验（如 Tailwind 类名规范、数据库迁移模式、测试策略），形成了知识在 Story 间的自然传递链。

### 5. 安全设计前置

CORE-006 的 API Key 加密存储实现质量高：
- AES-256-GCM 加密、随机 nonce、Base64 编码
- 密钥文件 600 权限
- 自动明文迁移
- 内存安全清除（best effort）
- 日志脱敏显示

作为第一个 Epic 就将安全性纳入，为后续功能建立了安全基线。

---

## 三、遇到的挑战 (Challenges Encountered)

### 1. xcap 条件编译问题

**问题**: `xcap` 截图库在 CI 环境（无桌面环境）下编译失败。原始的 `[target.'cfg(...)'.dependencies]` 配置无法正确解析 feature 条件。

**解决**: CORE-002 将 xcap 改为 optional 依赖 + feature 声明：
```toml
xcap = { version = "0.9", optional = true }
[features]
screenshot = ["dep:xcap"]
```

**影响**: 此问题在后续 CORE-003 中也需要处理（条件编译 auto_perception 模块），说明基础依赖配置问题应尽早在 Epic 开始前解决。

### 2. 前端测试环境不稳定

**问题**: 异步操作测试在 CI 环境偶尔超时，原始的多次 `nextTick` 等待策略不可靠。

**解决**: CORE-002 引入 `waitFor` 条件轮询辅助函数，基于条件而非固定次数等待：
```javascript
async function waitFor(fn, timeout = 1000) {
  const start = Date.now();
  while (Date.now() - start < timeout) {
    if (fn()) return true;
    await new Promise(resolve => setTimeout(resolve, 10));
  }
  return false;
}
```

### 3. 多 Agent 模型一致性

CORE epic 使用了多个 Agent 模型（BMAD dev-story Workflow、Claude Opus 4.6、Claude GLM-5），不同模型的代码风格和文档质量存在差异：
- CORE-005 的 Dev Agent Record 留有模板占位符（`{{agent_model_name_version}}`），说明某些模型对模板的遵循不够严格
- 不同模型产生的测试粒度不一致

### 4. 数据库迁移策略

**问题**: 多个 Story 都需要修改 settings 表（CORE-003 添加字段、CORE-006 修改字段含义），迁移脚本互相依赖。

**解决**: 采用幂等迁移模式 `let _ = conn.execute(...)` 忽略「列已存在」错误。但随着字段增多（截至 CORE-006，Settings 已有 15+ 字段），这种 ALTER TABLE 方式的可维护性下降。

---

## 四、关键经验教训 (Key Lessons Learned)

### 1. UI 优化 Story 应作为基础性投资

CORE-001 的 UI 规范化投入（3pts）为后续 5 个 Story 节省了大量 UI 决策时间。建议未来 Epic 的第一个 Story 也遵循「先规范，后功能」模式。

### 2. 错误处理应尽早系统化

CORE-004 的 Toast 通知系统和错误分类机制是全局性基础设施。如果在 CORE-001/002 就建立统一的错误处理，可以避免后续 Story 重复实现临时方案。

**建议**: 将「统一错误处理」作为 Epic 前置任务（准备 Sprint），而非 Epic 中期 Story。

### 3. 条件编译配置应标准化

xcap 条件编译问题在 CORE-002 和 CORE-003 中反复出现。建议在 Epic 启动前就统一处理所有平台相关的条件编译配置。

### 4. Settings 表的扩展需要更好的迁移策略

当前的 ALTER TABLE + 幂等忽略模式在 Story 数量增多后变得脆弱。建议：
- 引入版本号迁移（schema_version 表）
- 或使用 Tauri 自带的数据库迁移机制

### 5. Story Intelligence 传承是有效的

在 Story 文件中明确记录「从前序 Story 学到的经验」，使得知识在无需人工干预的情况下自动传递。这是 BMAD 工作流的一个优秀实践。

---

## 五、技术债务 (Technical Debt)

| 项目 | 描述 | 来源 Story | 优先级 | 状态 |
|-----|------|-----------|--------|------|
| 前端测试 CI 环境 | vitest 依赖在 CI 环境的安装和运行需要验证 | CORE-002 | Medium | 已在后续 Sprint 解决 |
| CORE-005 文档不完整 | Dev Agent Record 包含模板占位符 | CORE-005 | Low | 不影响功能 |
| Settings 字段膨胀 | settings 表已有 15+ 字段，ALTER TABLE 迁移不可持续 | CORE-003/006 | Medium | 未来 Epic 需关注 |
| 全局状态测试隔离 | `AUTO_CAPTURE_RUNNING` 全局 AtomicBool 导致测试间偶发干扰 | CORE-005 | Low | CI 偶尔出现 flaky test |

---

## 六、对后续 Epic 的影响分析

### 对 Epic 2 (智能捕获优化) 的影响

| 贡献 | 说明 |
|------|------|
| xcap 条件编译模式 | CORE-002 建立的 optional dependency 模式被 SMART 系列直接复用 |
| Toast 错误处理 | CORE-004 的错误处理基础设施被所有后续 Story 使用 |
| Settings 扩展模式 | CORE-003 建立的「ALTER TABLE + 结构体同步」模式被 SMART-002/003 继承 |
| 测试策略 | CORE 建立的「每个 AC 对应测试用例」模式成为后续 Story 标准 |

### 对 Epic 3 (AI 能力提升) 的影响

| 贡献 | 说明 |
|------|------|
| API Key 加密 | CORE-006 的加密模块直接被 AI-001/005 使用 |
| 日报模板框架 | CORE-003 的模板系统被 AI-003 扩展 |
| 多模型配置基础 | CORE-003 的 summary_model_name 分离为 AI-001 的多模型架构奠定基础 |

---

## 七、团队协作亮点

1. **Story 间知识自动传递**: Previous Story Intelligence 机制确保每个 Story 都站在前序 Story 的肩膀上
2. **代码审查效果显著**: 所有 Story 均通过代码审查，发现的问题都是有价值的改进点
3. **TDD 文化建立**: 从 CORE-001 的 12 个测试到 CORE-006 的 157 个总测试，测试覆盖持续增长
4. **安全意识前置**: 在第一个 Epic 就完成了 API Key 加密（通常被推迟到后期）

---

## 八、量化指标

| 指标 | 数值 |
|------|------|
| 总 Story Points | 19 |
| 完成率 | 100% (6/6) |
| 总测试用例 | ~157 Rust + ~92 Frontend = ~249 |
| 代码审查通过率 | 100% |
| 生产事故 | 0 |
| 新增前端文件 | ~8 |
| 修改后端文件 | ~12 |
| 新增 Tauri 命令 | 8+ |
| 数据库 Schema 变更 | 2 个新字段 + 加密逻辑 |

---

## 九、Action Items

| # | 项目 | 类型 | 负责人 | 优先级 | 备注 |
|---|-----|------|--------|--------|------|
| 1 | 考虑引入数据库版本迁移机制 | 技术改进 | Charlie (Senior Dev) | Medium | 替代当前的 ALTER TABLE + 幂等忽略模式 |
| 2 | 标准化 Agent 模型输出质量 | 流程改进 | Bob (Scrum Master) | Low | 确保 Dev Agent Record 不遗留模板占位符 |
| 3 | 全局状态测试隔离策略 | 测试改进 | Dana (QA Engineer) | Low | 解决 AtomicBool 导致的 flaky test |

---

## 十、总结

CORE Epic 作为项目的第一个 Epic，成功完成了以下目标：

1. **功能完善**: 6 个 Story 覆盖了设置优化、画廊增强、模板定制、错误处理、托盘菜单、安全加密
2. **规范建立**: 建立了 UI 设计语言、测试策略、错误处理框架、安全基线
3. **知识传递**: Previous Story Intelligence 机制确保经验在 Story 间自然流动
4. **质量保障**: 100% 代码审查通过率、~249 个测试用例、0 生产事故

作为 MVP+ 强化阶段，CORE Epic 为后续 4 个 Epic（SMART、AI、DATA、REPORT）奠定了坚实的技术基础和开发规范。

---

**复盘执行者**: Claude Opus 4.6
**复盘日期**: 2026-03-15
