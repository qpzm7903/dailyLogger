# Story 4.3: 标签系统

Status: ready-for-dev

## Story

As a DailyLogger 用户,
I want 手动给记录打标签便于检索,
so that 我可以按工作主题、项目或类别组织记录，快速定位相关内容.

## Acceptance Criteria

1. **AC1: 添加/编辑标签**
   - Given 用户查看记录列表, When 点击记录的标签区域, Then 可添加新标签或编辑现有标签
   - 标签输入支持自由文本，自动提示已存在的标签
   - 支持为同一记录添加多个标签
   - 标签颜色自动生成或用户可选

2. **AC2: 标签云浏览**
   - Given 用户打开标签浏览视图, When 查看标签云, Then 显示所有标签及其使用频率
   - 标签大小反映使用次数
   - 点击标签可查看关联的所有记录
   - 支持标签重命名和删除

3. **AC3: 多标签组合筛选**
   - Given 用户在筛选面板, When 选择多个标签, Then 返回同时包含所有选中标签的记录
   - 支持 AND/OR 逻辑切换
   - 筛选结果实时更新

## Tasks / Subtasks

- [ ] Task 1: 数据库 Schema 迁移 (AC: 1)
  - [ ] 1.1 创建 `tags` 表 (id, name, color, created_at)
  - [ ] 1.2 创建 `record_tags` 关联表 (record_id, tag_id)
  - [ ] 1.3 添加迁移逻辑到 `init_database`
  - [ ] 1.4 编写 Schema 迁移单元测试

- [ ] Task 2: 后端标签 API (AC: 1, 2, 3)
  - [ ] 2.1 实现 `add_tag_to_record(record_id, tag_name)` 函数
  - [ ] 2.2 实现 `remove_tag_from_record(record_id, tag_id)` 函数
  - [ ] 2.3 实现 `get_all_tags()` 函数 (返回标签列表及使用计数)
  - [ ] 2.4 实现 `get_records_by_tags(tag_ids[], logic)` 函数
  - [ ] 2.5 实现 `update_tag(tag_id, new_name, new_color)` 函数
  - [ ] 2.6 实现 `delete_tag(tag_id)` 函数
  - [ ] 2.7 在 `main.rs` 注册所有 Tauri 命令
  - [ ] 2.8 添加单元测试覆盖 CRUD 操作

- [ ] Task 3: 前端标签组件 (AC: 1)
  - [ ] 3.1 创建 `TagBadge.vue` 组件 (单标签显示)
  - [ ] 3.2 创建 `TagInput.vue` 组件 (标签输入/编辑)
  - [ ] 3.3 实现标签自动补全 (从已有标签提示)
  - [ ] 3.4 在记录列表项中集成 TagBadge 显示

- [ ] Task 4: 标签云浏览 (AC: 2)
  - [ ] 4.1 创建 `TagCloud.vue` 组件
  - [ ] 4.2 实现标签大小映射 (使用次数 → 字体大小)
  - [ ] 4.3 实现标签点击筛选记录
  - [ ] 4.4 添加标签重命名/删除功能

- [ ] Task 5: 多标签筛选 (AC: 3)
  - [ ] 5.1 创建 `TagFilter.vue` 组件
  - [ ] 5.2 实现 AND/OR 逻辑切换 UI
  - [ ] 5.3 集成到主界面记录列表上方
  - [ ] 5.4 添加组件测试

## Dev Notes

### 架构约束

1. **数据库操作**: 使用现有的 `DB_CONNECTION` 全局 Mutex，不创建新的数据库连接
2. **Tauri 命令**: 所有新命令必须在 `main.rs` 的 `generate_handler![]` 中注册
3. **前端风格**: 使用 TailwindCSS，遵循 `bg-dark`、`bg-darker`、`text-primary` 主题色
4. **标签颜色**: 使用预定义颜色数组，避免复杂的颜色选择器

### 数据库 Schema 设计

**新增表结构** (添加到 `init_database`):

```sql
-- 标签表
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT DEFAULT '#3b82f6',  -- 默认蓝色
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 记录-标签关联表 (多对多)
CREATE TABLE IF NOT EXISTS record_tags (
    record_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (record_id, tag_id),
    FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- 索引优化
CREATE INDEX IF NOT EXISTS idx_record_tags_tag_id ON record_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);
```

### Rust 数据结构

```rust
// memory_storage/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub created_at: String,
    pub usage_count: Option<i64>,  // 用于标签云显示
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordWithTags {
    pub record: Record,
    pub tags: Vec<Tag>,
}
```

### API 设计

**Tauri 命令** (`memory_storage/mod.rs`):

```rust
// 标签 CRUD
#[command]
pub async fn get_all_tags() -> Result<Vec<Tag>, String>

#[command]
pub async fn add_tag_to_record(record_id: i64, tag_name: String) -> Result<Tag, String>

#[command]
pub async fn remove_tag_from_record(record_id: i64, tag_id: i64) -> Result<(), String>

#[command]
pub async fn update_tag(tag_id: i64, name: Option<String>, color: Option<String>) -> Result<(), String>

#[command]
pub async fn delete_tag(tag_id: i64) -> Result<(), String>

// 筛选查询
#[command]
pub async fn get_records_by_tags(
    tag_ids: Vec<i64>,
    logic: String  // "AND" | "OR"
) -> Result<Vec<RecordWithTags>, String>

// 获取记录的标签
#[command]
pub async fn get_tags_for_record(record_id: i64) -> Result<Vec<Tag>, String>
```

### 标签自动补全实现

**SQL 查询** (获取所有标签名用于提示):

```sql
SELECT name FROM tags ORDER BY name;
```

**前端实现**: 使用 Vue 的 `<datalist>` 或自定义下拉组件

### 标签云大小计算

**字体大小映射算法**:

```javascript
// 根据使用次数计算字体大小 (相对大小)
function calculateTagSize(usageCount, minCount, maxCount) {
  const minSize = 12;  // px
  const maxSize = 24;  // px
  if (maxCount === minCount) return minSize;
  return minSize + ((usageCount - minCount) / (maxCount - minCount)) * (maxSize - minSize);
}
```

### 多标签筛选 SQL

**AND 逻辑** (记录包含所有选中标签):

```sql
SELECT r.* FROM records r
WHERE r.id IN (
    SELECT rt.record_id FROM record_tags rt
    WHERE rt.tag_id IN (?1, ?2, ?3)
    GROUP BY rt.record_id
    HAVING COUNT(DISTINCT rt.tag_id) = 3  -- 选中标签数量
)
ORDER BY r.timestamp DESC;
```

**OR 逻辑** (记录包含任意选中标签):

```sql
SELECT DISTINCT r.* FROM records r
JOIN record_tags rt ON r.id = rt.record_id
WHERE rt.tag_id IN (?1, ?2, ?3)
ORDER BY r.timestamp DESC;
```

### 前端组件结构

```
TagBadge.vue          -- 单标签显示组件
├─ span.tag-badge     -- 标签样式容器
│  ├─ span.name       -- 标签名
│  └─ button.remove   -- 删除按钮 (可选)

TagInput.vue          -- 标签输入组件
├─ div.tags-container -- 已选标签区
│  └─ TagBadge (v-for)
├─ input              -- 标签输入框
└─ datalist/ul        -- 自动补全列表

TagCloud.vue          -- 标签云浏览
├─ header             -- 标题 + 关闭按钮
├─ div.tag-cloud      -- 标签云容器
│  └─ span.tag (v-for, :style="fontSize")
└─ div.tag-actions    -- 重命名/删除操作

TagFilter.vue         -- 标签筛选组件
├─ div.selected-tags  -- 已选标签
├─ div.logic-toggle   -- AND/OR 切换
└─ button.clear       -- 清空筛选
```

### 项目结构 Notes

- 新组件位置: `src/components/Tag*.vue`
- 后端修改: `src-tauri/src/memory_storage/mod.rs`, `src-tauri/src/main.rs`
- 遵循现有命名规范: snake_case (Rust), PascalCase (Vue)
- 与 DATA-001/DATA-002 共享 UI 风格和布局模式

### 测试要求

**Rust 测试** (`memory_storage/mod.rs`):
- 测试添加标签到记录
- 测试重复添加同一标签 (应幂等)
- 测试删除标签同时清理关联
- 测试 AND/OR 筛选逻辑正确性
- 测试标签重命名唯一性约束

**前端测试** (Vitest):
- TagBadge 正确显示标签名和颜色
- TagInput 支持输入和自动补全
- TagCloud 大小映射正确
- 筛选切换触发正确 API 调用

### 预定义标签颜色

```javascript
const TAG_COLORS = [
  '#3b82f6', // blue
  '#10b981', // emerald
  '#f59e0b', // amber
  '#ef4444', // red
  '#8b5cf6', // violet
  '#ec4899', // pink
  '#06b6d4', // cyan
  '#84cc16', // lime
];
```

### 与其他 Story 的关系

- **DATA-001 (历史记录浏览)**: 标签系统需要在历史记录视图中显示标签
- **DATA-002 (全文搜索)**: 可考虑未来支持标签 + 关键词组合搜索
- **AI-004 (AI 分类标签生成)**: AI 自动生成标签后可复用本 Story 的标签系统

### References

- [Source: architecture.md#5.1] 数据库 schema 和索引设计模式
- [Source: architecture.md#6] API 端点设计模式
- [Source: PRD.md#11] 未来规划 - 标签系统 P2 优先级
- [Source: epics.md#Epic 4] 数据管理 Epic 上下文
- [Source: DATA-002.md] 类似功能实现模式参考

## Dev Agent Record

### Agent Model Used

(待实现时填写)

### Debug Log References

(待实现时填写)

### Completion Notes List

(待实现时填写)

### File List

(待实现时填写)