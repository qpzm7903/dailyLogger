# Story 10.4: 性能优化 - 数据库查询

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a DailyLogger user,
I want search and filter operations to return results instantly even with 1000+ records,
so that I can quickly find historical screenshots and entries without waiting.

**来源**: plan.md 未来规划 - 性能优化（大量截图时的流畅度）

## Background

当前数据库查询未针对大数据量场景优化。当用户有 1000+ 条记录时：
- 全文搜索没有索引，LIKE 查询全表扫描
- 日期范围筛选未使用索引
- 分页使用 OFFSET，大页码时性能差

**Epic 10 定位**：
```
Epic 10: 体验极致化
├── PERF-001: AI 配置完善（代理支持） ✅ 已完成
├── PERF-002: 新用户引导 ✅ 已完成
├── PERF-003: 性能优化 - 截图加载 ✅ 已完成
├── PERF-004: 性能优化 - 数据库查询 ← 当前
├── PERF-005: 多语言支持 (i18n)
└── PERF-006: 浅色主题支持
```

## Acceptance Criteria

1. **全文搜索性能**
   - Given 数据库有 1000+ 条记录
   - When 用户执行全文搜索
   - Then 搜索结果在 1 秒内返回
   - And 使用 SQLite FTS5 全文搜索索引

2. **日期筛选索引优化**
   - Given 用户按日期筛选记录
   - When 选择日期范围
   - Then 查询使用索引，响应时间 < 500ms
   - And 确保 idx_timestamp 索引存在且被正确使用

3. **游标分页优化**
   - Given 用户浏览历史记录
   - When 分页加载下一页
   - Then 使用游标分页（keyset pagination），避免 OFFSET 性能问题
   - And 支持跳转到任意页码（通过记录 ID 定位）

4. **会话查询优化**
   - Given 用户查看工作时段
   - When 加载今日会话列表
   - Then 使用 idx_sessions_date 索引
   - And 每个会话的截图数量统计使用预聚合或高效查询

## Tasks / Subtasks

- [ ] Task 1: 集成 SQLite FTS5 全文搜索 (AC: #1)
  - [ ] 分析现有搜索功能实现（memory_storage/mod.rs 中的查询）
  - [ ] 创建 FTS5 虚拟表用于 records.content 全文搜索
  - [ ] 实现 triggers 保持 FTS 索引与 records 表同步
  - [ ] 修改搜索查询使用 MATCH 而非 LIKE

- [ ] Task 2: 验证并优化日期索引 (AC: #2)
  - [ ] 检查 idx_timestamp 索引是否存在
  - [ ] 验证 EXPLAIN QUERY PLAN 确认索引被使用
  - [ ] 如需要，添加复合索引 (timestamp, session_id) 优化会话相关查询

- [ ] Task 3: 实现游标分页 (AC: #3)
  - [ ] 分析现有分页实现（get_today_records 等函数）
  - [ ] 将 OFFSET 分页改为 keyset 分页
  - [ ] 添加基于 ID 的高效跳页机制
  - [ ] 更新前端分页逻辑（如有）

- [ ] Task 4: 会话查询优化 (AC: #4)
  - [ ] 检查 sessions 表查询性能
  - [ ] 优化 get_today_sessions 相关查询
  - [ ] 考虑会话内截图数量的预聚合

- [ ] Task 5: 回归测试 (AC: all)
  - [ ] 测试全文搜索准确性（FTS vs LIKE 结果对比）
  - [ ] 测试日期筛选功能正常
  - [ ] 测试分页功能正常
  - [ ] 测试会话查询功能正常

## Dev Notes

### 关键架构约束

1. **SQLite 版本**: 项目使用 SQLite，需确认 FTS5 支持（SQLite 3.9.0+ 内置支持）
2. **向后兼容**: FTS 迁移需要处理已有数据
3. **Rust 后端**: 所有数据库操作在 `src-tauri/src/memory_storage/mod.rs`

### 文件树组件（需修改）

```
src-tauri/src/
├── memory_storage/
│   ├── mod.rs              # 修改：FTS 查询、游标分页
│   └── database.rs         # 可能需要：数据库迁移、FTS 表创建
src-tauri/src/
├── session_manager/
│   └── mod.rs              # 可能需要：优化会话查询
src/
├── components/
│   └── ScreenshotGallery.vue  # 可能需要：配合游标分页
```

### FTS5 实现方案

```rust
// 1. 创建 FTS5 虚拟表
CREATE VIRTUAL TABLE records_fts USING fts5(
    content,
    content='records',
    content_rowid='id'
);

// 2. 创建 triggers 保持同步
CREATE TRIGGER records_ai AFTER INSERT ON records BEGIN
    INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
END;

CREATE TRIGGER records_ad AFTER DELETE ON records BEGIN
    INSERT INTO records_fts(records_fts, rowid, content) VALUES('delete', old.id, old.content);
END;

CREATE TRIGGER records_au AFTER UPDATE ON records BEGIN
    INSERT INTO records_fts(records_fts, rowid, content) VALUES('delete', old.id, old.content);
    INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
END;

// 3. 迁移已有数据
INSERT INTO records_fts(rowid, content) SELECT id, content FROM records;
```

### 游标分页方案

传统 OFFSET 分页（性能问题）：
```sql
SELECT * FROM records ORDER BY timestamp DESC LIMIT 20 OFFSET 1000;
```

游标分页（高效）：
```sql
-- 首次加载
SELECT * FROM records ORDER BY id DESC LIMIT 20;

-- 下一页（传入 last_id）
SELECT * FROM records WHERE id < :last_id ORDER BY id DESC LIMIT 20;

-- 跳转到指定位置
SELECT * FROM records WHERE id < :anchor_id ORDER BY id DESC LIMIT :offset, 20;
```

### 索引验证查询

```sql
-- 检查现有索引
SELECT name, sql FROM sqlite_master WHERE type='index' AND tbl_name='records';

-- 验证索引使用
EXPLAIN QUERY PLAN SELECT * FROM records WHERE timestamp BETWEEN :start AND :end;
```

## Testing Requirements

1. **性能测试**：
   - 插入 1000+ 条测试数据
   - 测量全文搜索响应时间（目标 < 1s）
   - 测量日期筛选响应时间（目标 < 500ms）
   - 测量游标分页性能

2. **功能测试**：
   - FTS 搜索结果与 LIKE 搜索结果对比（召回率）
   - 分页跳转功能正常
   - 会话查询功能正常

3. **回归测试**：
   - 新增 FTS 记录不影响现有 CRUD
   - 日期筛选结果正确
   - 分页不遗漏记录

## References

- [Source: _bmad-output/planning-artifacts/architecture.md#Section 5] - 数据库设计和索引
- [Source: _bmad-output/planning-artifacts/epics.md#Epic 10] - Story 10.4 原始需求
- [Source: src-tauri/src/memory_storage/mod.rs] - 现有数据库操作实现
- [Source: src-tauri/src/session_manager/mod.rs] - 会话管理模块

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
