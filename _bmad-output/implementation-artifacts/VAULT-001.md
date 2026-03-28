# Story 12.1: VAULT-001 - 多 Obsidian Vault 支持

Status: in-progress (Task 1 done, Task 2 (backend vault param) done, Task 3 (auto-detect) done, Tasks 4-5 (UI) remaining, Task 6 (tests) remaining)

## Story

As a software developer who works on multiple projects,
I want to output daily reports to different Obsidian Vaults based on the project I'm working on,
So that I can keep work records organized by project context.

## Acceptance Criteria

1. **Given** 用户配置了多个 Obsidian Vault 路径，**When** 生成日报，**Then** 可以选择目标 Vault
2. **Given** 用户在设置中启用了项目检测（如通过窗口标题），**When** 自动生成日报，**Then** 自动输出到对应项目的 Vault
3. **Given** 用户手动选择 Vault，**When** 点击生成日报，**Then** 输出到用户指定的 Vault
4. **Given** 单个 Vault 配置，**When** 使用应用，**Then** 行为与之前一致（向后兼容）

## Tasks / Subtasks

- [x] Task 1: 扩展 ObsidianVault 数据结构，添加项目检测配置 (AC: #2)
  - [x] 1.1 在 `ObsidianVault` struct 中添加 `window_patterns: Vec<String>` 字段，用于匹配窗口标题
  - [x] 1.2 添加 `auto_detect_vault_by_window: bool` 设置项到 settings 表
  - [x] 1.3 更新 Rust 端 `ObsidianVault` struct 和序列化逻辑
  - [x] 1.4 更新前端 `ObsidianVault` TypeScript 接口和 OutputSettings.vue

- [x] Task 2: Rust 后端 - 修改 generate_daily_summary 支持 vault 参数 (AC: #1, #3)
  - [x] 2.1 修改 `generate_daily_summary` 命令签名，添加可选参数 `vault_name: Option<String>`
  - [x] 2.2 如果指定了 `vault_name`，使用对应 Vault 的 path；否则使用默认逻辑
  - [x] 2.3 添加 `get_vault_by_name(&self, name: &str)` 和 `get_vault_by_window_title(&self, title: &str)` 辅助方法
  - [x] 2.4 在 main.rs 的 `generate_handler![]` 中注册更新后的命令

- [x] Task 3: Rust 后端 - 实现基于窗口标题的自动 Vault 选择 (AC: #2)
  - [x] 3.1 在 `generate_daily_summary` 中，如果 `auto_detect_vault_by_window` 为 true 且未指定 vault_name
  - [x] 3.2 调用 `window_info::get_active_window()` 获取当前窗口标题
  - [x] 3.3 遍历 `obsidian_vaults`，查找 `window_patterns` 中有匹配项的 Vault
  - [x] 3.4 如果找到匹配，使用该 Vault；否则回退到默认 Vault

- [ ] Task 4: 前端 - 添加 Vault 选择器 UI (AC: #1, #3)
  - [ ] 4.1 在 ReportDropdown.vue 中，当有多个 Vault 时，显示 Vault 选择下拉框
  - [ ] 4.2 调用 `generate_daily_summary(vaultName?)` 时传入选中的 vault name
  - [ ] 4.3 如果只有一个 Vault，保持现有行为不变（向后兼容）

- [ ] Task 5: 前端 - 添加项目检测开关 UI (AC: #2)
  - [ ] 5.1 在 OutputSettings.vue 中添加"自动根据窗口标题选择 Vault"开关
  - [ ] 5.2 在 Vault 编辑区域，为每个 Vault 添加"窗口标题匹配模式"输入框（多个用逗号分隔）
  - [ ] 5.3 保存设置时，将配置序列化为 JSON 存储

- [ ] Task 6: 单元测试 (AC: All)
  - [ ] 6.1 测试 `get_vault_by_name` - 正常查找、不存在查找、空名称
  - [ ] 6.2 测试 `get_vault_by_window_title` - 精确匹配、部分匹配、多模式匹配、无匹配
  - [ ] 6.3 测试 `generate_daily_summary` 带 vault_name 参数的各种场景
  - [ ] 6.4 测试自动检测逻辑 - 开启/关闭、匹配/不匹配

## Dev Notes

### Architecture Context

**关键架构决策**:
- 复用现有的 `ObsidianVault` 数据结构，向后兼容 `is_default` 字段
- 使用 `window_info::matches_any()` 进行窗口标题模式匹配（已在 SMART-001 中使用）
- Vault 选择通过 Tauri command 参数传递，不改变现有 `generate_daily_summary()` 的基本流程
- 项目检测仅在用户明确启用且未手动选择 Vault 时生效

**必须遵循的代码模式** [Source: architecture.md]:
- Tauri Command: `#[command]` + async
- 错误处理: `Result<T, String>` + `.map_err(|e| e.to_string())`
- 数据库访问: 使用全局 `DB_CONNECTION` Mutex
- 序列化: `serde_json` 用于 `obsidian_vaults` JSON 字段

### Key Existing Code to Reuse

**window_info/mod.rs** - 复用以下函数:
- `get_active_window()` - 获取当前活动窗口信息
- `matches_any(text: &str, patterns: &[String]) -> bool` - 模式匹配（大小写不敏感，包含匹配）

**memory_storage/mod.rs** - 复用以下结构:
- `ObsidianVault` struct - 扩展添加 `window_patterns` 字段
- `Settings` struct - 添加 `auto_detect_vault_by_window` 字段
- `get_obsidian_output_path()` - 参考其查找默认 Vault 的逻辑

**synthesis/mod.rs** - 修改以下函数:
- `generate_daily_summary()` - 添加可选 `vault_name` 参数

### Database Migration

在 `init_database()` 中添加:
```rust
let _ = conn.execute(
    "ALTER TABLE settings ADD COLUMN auto_detect_vault_by_window INTEGER DEFAULT 0",
    [],
);
```

### Data Structures

**扩展 ObsidianVault**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianVault {
    pub name: String,
    pub path: String,
    pub is_default: bool,
    pub window_patterns: Option<Vec<String>>, // 新增：匹配窗口标题的模式
}
```

**TypeScript 扩展**:
```typescript
interface ObsidianVault {
  name: string
  path: string
  is_default?: boolean
  window_patterns?: string[]  // 新增
}
```

### Settings 扩展

```rust
// memory_storage/mod.rs Settings struct 中添加
pub auto_detect_vault_by_window: Option<bool>, // 新增
```

### Vault Selection Logic

```rust
impl Settings {
    /// Get vault by name
    pub fn get_vault_by_name(&self, name: &str) -> Option<&ObsidianVault> {
        if let Some(ref vaults_json) = self.obsidian_vaults {
            if let Ok(vaults) = serde_json::from_str::<Vec<ObsidianVault>>(vaults_json) {
                return vaults.iter().find(|v| v.name == name);
            }
        }
        None
    }

    /// Get vault by active window title (for auto-detection)
    pub fn get_vault_by_window_title(&self, title: &str) -> Option<&ObsidianVault> {
        if let Some(ref vaults_json) = self.obsidian_vaults {
            if let Ok(vaults) = serde_json::from_str::<Vec<ObsidianVault>>(vaults_json) {
                // First try to find a vault with matching window pattern
                for vault in &vaults {
                    if let Some(ref patterns) = vault.window_patterns {
                        if window_info::matches_any(title, patterns) {
                            return Some(vault);
                        }
                    }
                }
            }
        }
        None
    }

    /// Get effective output vault
    pub fn get_effective_vault(&self, vault_name: Option<&str>, auto_detect: bool) -> Result<String, String> {
        // 1. If explicitly specified, use that vault
        if let Some(name) = vault_name {
            if let Some(vault) = self.get_vault_by_name(name) {
                return Ok(vault.path.clone());
            }
            return Err(format!("Vault '{}' not found", name));
        }

        // 2. If auto-detect is enabled, try to detect by window
        if auto_detect {
            if let Some(window) = window_info::get_active_window() {
                if let Some(vault) = self.get_vault_by_window_title(&window.title) {
                    tracing::info!("Auto-detected vault '{}' for window '{}'", vault.name, window.title);
                    return Ok(vault.path.clone());
                }
            }
        }

        // 3. Fall back to default vault (existing logic)
        self.get_obsidian_output_path()
    }
}
```

### Frontend Vault Selector UI

**ReportDropdown.vue 修改**:
```vue
<!-- Vault selector dropdown (shown when multiple vaults exist) -->
<select v-if="vaults.length > 1" v-model="selectedVault" class="...">
  <option value="">默认 Vault</option>
  <option v-for="vault in vaults" :key="vault.name" :value="vault.name">
    {{ vault.name }}
  </option>
</select>

<!-- Pass selected vault to backend -->
const result = await invoke<string>('generate_daily_summary', {
  vaultName: selectedVault || null
})
```

### OutputSettings.vue 增强

```vue
<!-- 每个 Vault 的窗口匹配模式 -->
<div v-for="(vault, index) in vaults" :key="index" class="vault-item">
  <!-- 现有 UI... -->
  <!-- 新增: 窗口模式输入 -->
  <input
    v-model="vault.window_patterns"
    type="text"
    placeholder="窗口标题匹配模式，如: VS Code, project-A"
    class="..."
  />
  <span class="text-xs text-gray-500">多个模式用逗号分隔</span>
</div>

<!-- 自动检测开关 -->
<div class="flex items-center gap-2">
  <input
    v-model="settings.auto_detect_vault_by_window"
    type="checkbox"
    id="auto-detect-vault"
  />
  <label for="auto-detect-vault" class="text-sm text-gray-300">
    根据窗口标题自动选择 Vault
  </label>
</div>
```

### Project Structure Notes

**需要修改的文件**:
- `src-tauri/src/memory_storage/mod.rs` - 扩展 ObsidianVault、Settings、添加 helper 方法
- `src-tauri/src/memory_storage/settings.rs` - 更新序列化/反序列化
- `src-tauri/src/synthesis/mod.rs` - 修改 generate_daily_summary 签名和逻辑
- `src-tauri/src/main.rs` - 注册命令参数变更
- `src-tauri/src/window_info/mod.rs` - 复用 matches_any 函数
- `src/types/tauri.ts` - 扩展 TypeScript 类型
- `src/components/ReportDropdown.vue` - 添加 Vault 选择器
- `src/components/settings/OutputSettings.vue` - 添加项目检测 UI

### Testing Requirements

**必须测试的场景**:
1. **get_vault_by_name**:
   - 存在指定名称的 Vault → 返回 Some
   - 不存在指定名称 → 返回 None
   - 空名称 → 返回 None

2. **get_vault_by_window_title**:
   - 窗口标题精确匹配 Vault 的 window_patterns → 返回该 Vault
   - 窗口标题部分匹配 Vault 的 window_patterns → 返回该 Vault
   - 多个 Vault 都能匹配 → 返回第一个匹配
   - 没有 Vault 匹配 → 返回 None

3. **generate_daily_summary 带参数**:
   - 指定存在的 vault_name → 使用该 Vault 的 path
   - 指定不存在的 vault_name → 返回错误
   - vault_name=None 且 auto_detect=false → 使用默认 Vault
   - vault_name=None 且 auto_detect=true 且有匹配窗口 → 使用匹配的 Vault
   - vault_name=None 且 auto_detect=true 但无匹配窗口 → 使用默认 Vault

4. **向后兼容**:
   - 只有一个 Vault 时，行为与之前完全一致
   - obsidian_vaults 为空/null 时，使用 legacy obsidian_path

### References

- [Source: architecture.md#2.2] - 后端模块架构
- [Source: architecture.md#3.2] - 日报生成流程
- [Source: architecture.md#5.1] - 数据库 Schema
- [Source: PRD.md#11] - FR11 多 Obsidian Vault 支持
- [Source: epics.md#Epic 12] - Epic 12 详细描述
- [Source: memory_storage/mod.rs] - ObsidianVault, Settings, get_obsidian_output_path
- [Source: window_info/mod.rs] - get_active_window, matches_any
- [Source: synthesis/mod.rs] - generate_daily_summary
- [Source: OutputSettings.vue] - Vault 管理 UI（现有实现参考）
- [Source: ReportDropdown.vue] - 报告生成按钮 UI

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

