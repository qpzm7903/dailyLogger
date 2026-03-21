# Story 4.6: 多 Obsidian Vault 支持

Status: done

## Story

As a DailyLogger 用户,
I want 支持配置多个 Obsidian Vault,
so that 我可以为不同项目配置不同的输出目录，实现工作记录的灵活分类管理.

## Acceptance Criteria

1. **AC1: 添加/删除 Vault**
   - Given 用户打开设置界面, When 添加新 Vault, Then 输入名称和路径后保存到配置
   - 支持删除已有的 Vault 配置

2. **AC2: 设置默认 Vault**
   - Given 用户有多个 Vault, When 点击星标按钮, Then 设置该 Vault 为默认输出目录
   - 默认 Vault 用于日报/周报输出

3. **AC3: Vault 输出路径选择**
   - Given 用户生成日报, When 系统查找输出路径, Then 优先使用默认 Vault 路径
   - 如果没有配置 Vault，回退到 legacy `obsidian_path`

4. **AC4: 数据迁移兼容**
   - Given 用户有旧的 `obsidian_path` 配置, When 首次加载, Then 自动迁移为默认 Vault
   - 保持向后兼容

## Tasks / Subtasks

- [x] Task 1: 后端数据模型扩展 (AC: 3, 4)
  - [x] 1.1 在 Settings 结构体添加 `obsidian_vaults` 字段 (JSON 格式)
  - [x] 1.2 创建 `ObsidianVault` 结构体 (name, path, is_default)
  - [x] 1.3 实现 `get_obsidian_output_path()` 方法，优先使用 vaults
  - [x] 1.4 添加数据库迁移

- [x] Task 2: 前端 Vault 管理 UI (AC: 1, 2)
  - [x] 2.1 在 `OutputSettings.vue` 添加 Vault 列表显示
  - [x] 2.2 实现添加 Vault 表单
  - [x] 2.3 实现删除 Vault 按钮
  - [x] 2.4 实现设置默认 Vault (星标按钮)
  - [x] 2.5 实现 legacy `obsidian_path` 自动迁移逻辑

- [x] Task 3: 集成测试 (AC: 全部)
  - [x] 3.1 添加前端组件测试
  - [x] 3.2 更新 SettingsModal 集成

## Dev Notes

### 架构约束

1. **数据存储**: `obsidian_vaults` 使用 JSON 格式存储在 settings 表
2. **向后兼容**: 必须支持 legacy `obsidian_path` 字段
3. **Tauri 命令**: 复用现有 `get_settings` / `save_settings`

### 数据结构

```rust
// src-tauri/src/memory_storage/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianVault {
    pub name: String,
    pub path: String,
    pub is_default: bool,
}

// Settings 字段
pub obsidian_vaults: Option<String>, // JSON: Vec<ObsidianVault>
```

### 关键实现位置

- **Settings 结构体**: `src-tauri/src/memory_storage/mod.rs:75`
- **ObsidianVault 结构体**: `src-tauri/src/memory_storage/mod.rs:156-165`
- **get_obsidian_output_path()**: `src-tauri/src/memory_storage/mod.rs:167-193`
- **数据库迁移**: `src-tauri/src/memory_storage/schema.rs:223`
- **前端 Vault UI**: `src/components/settings/OutputSettings.vue:3-100`
- **Vault 测试**: `src/components/settings/__tests__/OutputSettings.test.ts:142-190`

### 前端组件结构

```
OutputSettings.vue
├── Obsidian Vaults section
│   ├── Vault list (v-for)
│   │   ├── Star button (set default)
│   │   ├── Name + Path display
│   │   └── Remove button
│   ├── Empty state message
│   └── Add vault form
│       ├── Name input
│       ├── Path input
│       └── Add button
```

### 输出路径选择逻辑

```rust
// synthesis/mod.rs 中调用
let obsidian_path = settings.get_obsidian_output_path()?;
// 1. 检查 obsidian_vaults 中的 default vault
// 2. 如果没有 default，使用第一个 vault
// 3. 如果没有 vaults，回退到 obsidian_path
// 4. 如果都没有配置，返回错误
```

### References

- [Source: src-tauri/src/memory_storage/mod.rs:156-193] ObsidianVault 结构体和路径选择逻辑
- [Source: src/components/settings/OutputSettings.vue:3-100] Vault 管理 UI
- [Source: PRD.md#11] 多 Obsidian Vault 支持 P2 优先级
- [Source: architecture.md#5.2] settings 表结构

## Dev Agent Record

### Agent Model Used

(实现时未记录 - 功能在开发其他 story 时顺便实现)

### Debug Log References

无

### Completion Notes List

- 功能已完整实现：添加/删除/设置默认 Vault
- 后端支持 legacy `obsidian_path` 自动迁移
- 前端测试覆盖完整：添加/删除/设置默认

### File List

**已实现文件：**
- `src-tauri/src/memory_storage/mod.rs` - ObsidianVault 结构体 + get_obsidian_output_path()
- `src-tauri/src/memory_storage/schema.rs` - obsidian_vaults 字段迁移
- `src-tauri/src/memory_storage/settings.rs` - 字段读写
- `src/components/settings/OutputSettings.vue` - Vault 管理 UI
- `src/components/settings/__tests__/OutputSettings.test.ts` - 测试

## Code Review Summary

**Review Date**: 2026-03-21
**Result**: ✅ PASS (Implementation Verified)

### AC Verification

| AC | Status | Evidence |
|----|--------|----------|
| AC1: 添加/删除 Vault | ✅ | OutputSettings.vue:262-277 |
| AC2: 设置默认 Vault | ✅ | OutputSettings.vue:280-284 |
| AC3: 输出路径选择 | ✅ | memory_storage/mod.rs:167-193 |
| AC4: 数据迁移兼容 | ✅ | SettingsModal.vue:477-480 |

### Test Results

- Frontend: 191+ tests passing
- Vault management tests: 5 tests passing

### Change Log

- 2026-03-21: Story documentation created - feature was already implemented