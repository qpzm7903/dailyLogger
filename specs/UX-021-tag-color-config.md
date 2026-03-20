# UX-021 标签颜色后端可配置

**版本**: v1.42.0
**优先级**: MEDIUM

## 功能需求

将当前前端硬编码的标签颜色映射移到后端（SQLite），允许用户或系统为每个标签指定颜色。前端从后端读取颜色配置，消除硬编码颜色数组，支持未来的用户自定义标签颜色功能。

**当前问题**:
- 标签颜色在前端 JS/TS 中硬编码（如 `const TAG_COLORS = ['blue', 'green', ...]`）
- 新增标签时颜色分配逻辑随意（如按索引取模）
- 用户无法自定义标签颜色
- 前端和后端对标签颜色的理解不一致

## 不在范围内

- 不实现用户通过 UI 修改标签颜色（属于 v1.43.0+ 功能）
- 不修改标签的创建和删除逻辑
- 不支持渐变色或自定义 CSS 颜色（仅支持预设调色盘）

## 接口定义

### 数据库 Schema 变更

```sql
-- 新增列到 records 表（或独立 tags 表）
-- 方案 A：在 records 表 content JSON 中记录标签颜色（无 schema 变更）
-- 方案 B：新增 tags 表（推荐，独立管理）

CREATE TABLE IF NOT EXISTS tags (
  id   INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  color TEXT NOT NULL DEFAULT 'blue'   -- 预设颜色 ID
);
```

### 预设调色盘

```typescript
// 允许的颜色值（对应 Tailwind 颜色类）
type TagColor =
  | 'blue'    // bg-blue-500
  | 'green'   // bg-green-500
  | 'yellow'  // bg-yellow-500
  | 'red'     // bg-red-500
  | 'purple'  // bg-purple-500
  | 'pink'    // bg-pink-500
  | 'indigo'  // bg-indigo-500
  | 'gray'    // bg-gray-500
```

### Tauri 命令

```rust
// 新增命令
#[tauri::command]
fn get_tag_colors() -> Result<HashMap<String, String>, String>
// 返回 { "工作": "blue", "学习": "green", ... }

#[tauri::command]
fn set_tag_color(tag: String, color: String) -> Result<(), String>
// 内部使用，暂不暴露 UI，供未来版本调用
```

### 颜色分配规则

新标签创建时，自动分配调色盘中使用次数最少的颜色（避免颜色集中），若所有颜色使用次数相同则循环分配。

## 验收条件（Given/When/Then）

### AC1 - 标签颜色从后端读取

- Given 后端 tags 表已记录标签颜色
- When 前端渲染标签列表
- Then 标签颜色与后端数据一致，不使用前端硬编码颜色

### AC2 - 新标签自动分配颜色

- Given 用户创建一个新标签"设计"
- When 标签保存到数据库
- Then 系统自动为该标签分配一个调色盘颜色，并存入 tags 表

### AC3 - 前端无硬编码颜色数组

- Given 重构完成
- When 检查前端源码
- Then 不存在 `TAG_COLORS = [...]` 形式的硬编码颜色数组

### AC4 - 旧数据兼容

- Given 数据库中已存在无颜色记录的旧标签
- When 前端请求标签颜色
- Then 旧标签使用默认颜色（'blue'），不报错

### AC5 - 颜色值在预设调色盘范围内

- Given 后端收到 `set_tag_color` 请求
- When 颜色值不在预设调色盘中（如 "hotpink"）
- Then 后端返回错误，拒绝写入

## 技术约束

- Schema 变更须提供 migration SQL（向前兼容旧数据库）
- Rust 测试须覆盖：颜色分配逻辑、颜色值校验、旧数据兼容
- 前端测试须覆盖：从后端读取颜色并渲染的逻辑
- `cargo test` 和 `npm run test` 全部通过
- 新 Tauri 命令须在 `main.rs` 的 `generate_handler![]` 中注册
