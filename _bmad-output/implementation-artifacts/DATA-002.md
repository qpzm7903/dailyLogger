# Story 4.2: 全文搜索功能

Status: done

## Story

As a DailyLogger 用户,
I want 全文搜索历史记录和截图分析内容,
so that 我可以快速定位特定的工作记录，无需逐条浏览.

## Acceptance Criteria

1. **AC1: 关键词搜索**
   - Given 用户打开搜索界面, When 输入关键词并提交, Then 返回包含该关键词的所有记录
   - 搜索范围包括：记录内容(content)、截图分析结果
   - 支持中英文搜索
   - 搜索框有清空按钮

2. **AC2: 搜索结果高亮显示**
   - Given 搜索结果已返回, When 用户查看结果列表, Then 关键词在内容中高亮显示
   - 高亮样式：黄色背景 + 深色文字
   - 上下文展示：显示关键词前后各 20 个字符

3. **AC3: 按相关性排序**
   - Given 搜索结果有多条, When 结果展示时, Then 按相关性分数降序排列
   - 相关性计算：标题匹配 > 内容匹配
   - 支持按时间排序切换

4. **AC4: 搜索性能**
   - Given 数据库有 1000 条记录, When 执行搜索, Then 响应时间 < 500ms
   - 使用 SQLite FTS5 全文索引

## Tasks / Subtasks

- [x] Task 1: 后端 FTS 索引与 API (AC: 1, 3, 4)
  - [x] 1.1 创建 FTS5 虚拟表迁移逻辑 (init_database 中添加)
  - [x] 1.2 实现 `search_records` 函数 (全文搜索 + 相关性排序)
  - [x] 1.3 在 `main.rs` 注册 `search_records` Tauri 命令
  - [x] 1.4 添加单元测试覆盖搜索边界条件

- [x] Task 2: 前端搜索组件 (AC: 1, 2)
  - [x] 2.1 创建 `SearchPanel.vue` 组件
  - [x] 2.2 实现搜索输入框与清空按钮
  - [x] 2.3 实现搜索结果列表与高亮显示
  - [x] 2.4 实现排序切换 (相关性/时间)

- [x] Task 3: 集成与入口 (AC: 全部)
  - [x] 3.1 在 `App.vue` 添加搜索入口按钮
  - [x] 3.2 集成 SearchPanel 到主界面
  - [x] 3.3 添加组件测试

## Dev Notes

### 架构约束

1. **数据库操作**: 使用现有的 `DB_CONNECTION` 全局 Mutex，不创建新的数据库连接
2. **Tauri 命令**: 所有新命令必须在 `main.rs` 的 `generate_handler![]` 中注册
3. **前端风格**: 使用 TailwindCSS，遵循 `bg-dark`、`bg-darker`、`text-primary` 主题色
4. **FTS 索引**: 使用 SQLite 内置 FTS5 扩展，无需额外依赖

### 关键代码参考

**现有 Record 结构体** (`src-tauri/src/memory_storage/mod.rs:99-106`):
```rust
pub struct Record {
    pub id: i64,
    pub timestamp: String,
    pub source_type: String,
    pub content: String,
    pub screenshot_path: Option<String>,
}
```

**FTS5 虚拟表创建** (添加到 `init_database`):
```sql
-- 全文搜索虚拟表
CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
    content,
    content='records',
    content_rowid='id',
    tokenize='unicode61'  -- 支持中文分词
);

-- 触发器：自动同步 FTS 索引
CREATE TRIGGER IF NOT EXISTS records_ai AFTER INSERT ON records BEGIN
    INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
END;

CREATE TRIGGER IF NOT EXISTS records_ad AFTER DELETE ON records BEGIN
    INSERT INTO records_fts(records_fts, rowid, content)
    VALUES ('delete', old.id, old.content);
END;

CREATE TRIGGER IF NOT EXISTS records_au AFTER UPDATE ON records BEGIN
    INSERT INTO records_fts(records_fts, rowid, content)
    VALUES ('delete', old.id, old.content);
    INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
END;
```

### 新增 API 设计

**Rust 端** (`memory_storage/mod.rs`):
```rust
/// 全文搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub record: Record,
    pub snippet: String,        // 高亮片段 (带 <mark> 标签)
    pub rank: f64,              // 相关性分数
}

/// 全文搜索记录
pub fn search_records(
    query: &str,               // 搜索关键词
    order_by: &str,            // "rank" | "time"
    limit: i64,                // 最大返回数
) -> Result<Vec<SearchResult>, String>
```

**Tauri 命令**:
```rust
#[command]
pub async fn search_records(
    query: String,
    order_by: Option<String>,  // 默认 "rank"
    limit: Option<i64>,        // 默认 50
) -> Result<Vec<SearchResult>, String>
```

**FTS5 搜索查询**:
```sql
-- 搜索 + 高亮 + 相关性排序
SELECT
    r.id, r.timestamp, r.source_type, r.content, r.screenshot_path,
    highlight(records_fts, 0, '<mark>', '</mark>') as snippet,
    bm25(records_fts) as rank
FROM records_fts
JOIN records r ON r.id = records_fts.rowid
WHERE records_fts MATCH ?1
ORDER BY rank  -- 或 r.timestamp DESC
LIMIT ?2;
```

### 前端组件结构

```
SearchPanel.vue
├─ Header: 搜索标题 + 关闭按钮
├─ SearchInput:
│  ├─ TextInput (搜索框)
│  └─ ClearButton (清空)
├─ SortToggle:
│  ├─ RelevanceButton (相关性)
│  └─ TimeButton (时间)
├─ ResultList:
│  ├─ ResultItem (v-for)
│  │  ├─ 时间戳
│  │  ├─ 来源标签
│  │  ├─ 高亮内容 (v-html)
│  │  └─ 相关性分数 (可选)
│  └─ LoadingIndicator
└─ EmptyState (无结果时显示)
```

### 高亮显示实现

**Rust 端**: 使用 FTS5 `highlight()` 函数，返回带 `<mark>` 标签的 HTML
**Vue 端**: 使用 `v-html` 渲染，添加 CSS 样式：

```css
.mark {
  background-color: #fef08a;  /* yellow-300 */
  color: #1e293b;             /* slate-800 */
  padding: 0 2px;
  border-radius: 2px;
}
```

### 项目结构 Notes

- 新组件位置: `src/components/SearchPanel.vue`
- 后端修改: `src-tauri/src/memory_storage/mod.rs`, `src-tauri/src/main.rs`
- 遵循现有命名规范: snake_case (Rust), PascalCase (Vue)
- 与 DATA-001 HistoryViewer 共享 UI 风格

### 测试要求

**Rust 测试** (`memory_storage/mod.rs`):
- 测试中文关键词搜索
- 测试空查询返回空结果
- 测试相关性排序正确性
- 测试高亮标签正确生成

**前端测试** (Vitest):
- 组件挂载时显示搜索框
- 输入触发搜索 API 调用
- 结果正确渲染高亮内容
- 排序切换重新搜索

### 性能考虑

- FTS5 索引使搜索复杂度降至 O(log n)
- `bm25()` 算法是业界标准的相关性评分
- 限制默认返回 50 条，避免大量数据传输
- 考虑防抖 (debounce) 输入，避免频繁 API 调用

### 与 DATA-001 的关系

- DATA-001 实现了 `HistoryViewer.vue` (历史记录浏览)
- DATA-002 的 `SearchPanel.vue` 是独立组件，可从主界面入口
- 未来可考虑在 HistoryViewer 中集成搜索功能
- 共享 `Record` 类型定义和 API 模式

### References

- [Source: architecture.md#5.1] 数据库 schema 和索引
- [Source: architecture.md#6] API 端点设计模式
- [Source: PRD.md#11] 未来规划 - 全文搜索 P2 优先级
- [Source: epics.md#Epic 4] 数据管理 Epic 上下文
- [SQLite FTS5 文档](https://www.sqlite.org/fts5.html)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

- 实现混合搜索策略：FTS5 用于英文关键词，LIKE 搜索作为中文/CJK 字符的回退
- unicode61 tokenizer 不支持中文分词，因此采用 CJK 字符检测和 LIKE 模式匹配

### Completion Notes List

- **2026-03-15 Task 1 完成**: 后端 FTS 索引与 API 已实现
  - 添加 SearchResult 结构体 (record, snippet, rank)
  - 创建 FTS5 虚拟表 `records_fts` 及同步触发器
  - 实现 `search_records_sync()` 支持 CJK 混合搜索
  - 注册 `search_records` Tauri 命令
  - 添加 7 个单元测试，覆盖中文搜索、英文搜索、排序、高亮等场景
  - 所有 170 个测试通过

- **2026-03-15 Task 2 完成**: 前端搜索组件已实现
  - 创建 SearchPanel.vue 组件
  - 实现搜索输入框与清空按钮
  - 实现搜索结果列表，使用 v-html 渲染高亮片段
  - 实现排序切换 (相关性/时间)，切换时自动重新搜索

- **2026-03-15 Task 3 完成**: 集成与入口已实现
  - 在 App.vue header 添加搜索按钮
  - 导入并集成 SearchPanel 组件
  - 所有 159 个前端测试通过

### File List

- `src-tauri/src/memory_storage/mod.rs` - FTS 表创建、search_records_sync()、SearchResult 结构体
- `src-tauri/src/main.rs` - 注册 search_records 命令
- `src/components/SearchPanel.vue` - 搜索面板组件
- `src/App.vue` - 集成搜索入口