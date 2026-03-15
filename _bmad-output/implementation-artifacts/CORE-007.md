# Story 1.7: 离线模式支持

Status: ready-for-dev

## Story

作为一个 DailyLogger 用户，
我希望在网络不可用时应用仍能正常截图并保存记录，在网络恢复后自动补充 AI 分析，
以便即使在离线环境下也不会丢失任何工作记录。

## Acceptance Criteria

### AC1 - 截图与分析解耦
- Given 自动截图定时触发
- When AI API 不可用（网络断开或 API 返回错误）
- Then 截图正常保存到文件系统，记录以 `pending_analysis` 状态存入数据库，不阻塞后续截图

### AC2 - 离线状态指示
- Given 应用正在运行
- When 网络连接断开或 API 不可达
- Then 前端显示离线状态指示器（如状态栏或图标），用户清楚知道当前离线

### AC3 - 自动重试队列
- Given 数据库中存在 `pending_analysis` 状态的记录
- When 网络恢复（API 可达）
- Then 后台自动按时间顺序逐条重试 AI 分析，使用指数退避策略（初始 30s，最大 5min），单条最多重试 3 次

### AC4 - 重试成功更新记录
- Given 某条 `pending_analysis` 记录的 AI 分析重试成功
- When 获取到分析结果
- Then 更新该记录的 `content` 字段为 AI 分析结果，状态变更为正常记录

### AC5 - 重试失败最终处理
- Given 某条 `pending_analysis` 记录已达最大重试次数（3 次）
- When 仍然分析失败
- Then 标记该记录为 `analysis_failed`，保留截图和时间戳，在前端截图画廊中以特殊样式展示（如灰色边框 + "分析失败" 标签）

### AC6 - 手动触发重试
- Given 前端截图画廊中显示有 `pending_analysis` 或 `analysis_failed` 的记录
- When 用户点击该记录的"重试分析"按钮
- Then 立即对该条记录发起一次 AI 分析请求

## Tasks / Subtasks

- [ ] Task 1: 数据库 Schema 扩展 (AC: 1, 3, 4, 5)
  - [ ] 在 records 表添加 `analysis_status` 字段 (TEXT, DEFAULT 'completed')
  - [ ] 在 records 表添加 `retry_count` 字段 (INTEGER, DEFAULT 0)
  - [ ] 实现幂等迁移（ALTER TABLE + 忽略已存在错误）
  - [ ] 添加查询函数 `get_pending_records_sync()` 返回 pending 记录
  - [ ] 编写迁移和查询的单元测试

- [ ] Task 2: 截图-分析解耦 (AC: 1)
  - [ ] 重构 `capture_and_store()` 中的分析流程
  - [ ] API 失败时：仍保存截图文件 + 插入 records（content 为空/占位，analysis_status='pending_analysis'）
  - [ ] API 成功时：正常流程不变（analysis_status='completed'）
  - [ ] 确保 screenshot_path 在任何情况下都正确保存
  - [ ] 编写分析失败场景的单元测试

- [ ] Task 3: API 连通性检测 (AC: 2)
  - [ ] 实现 Rust 后端 `check_api_connectivity()` 函数（HEAD 请求 + 3s 超时）
  - [ ] 注册为 Tauri 命令 `check_api_connectivity`
  - [ ] 前端添加定期检测逻辑（每 60s 检测一次，API 失败时缩短至 15s）
  - [ ] 编写连通性检测的单元测试

- [ ] Task 4: 前端离线状态指示 (AC: 2)
  - [ ] 在 App.vue 添加 `isApiOnline` 响应式状态
  - [ ] 添加离线指示器组件（状态栏位置，使用 TailwindCSS）
  - [ ] 在线：绿色圆点 + "在线"；离线：红色圆点 + "离线"
  - [ ] 编写前端状态指示器的测试

- [ ] Task 5: 后台重试引擎 (AC: 3, 4, 5)
  - [ ] 实现 `retry_pending_analyses()` 异步函数
  - [ ] 指数退避：初始 30s → 60s → 120s（最大 5min），单条最多 3 次
  - [ ] 成功时更新 content + analysis_status='completed' + 清零 retry_count
  - [ ] 失败且 retry_count >= 3 时标记 analysis_status='analysis_failed'
  - [ ] 在 `start_auto_capture()` 中增加重试任务（复用同一 tokio::spawn）
  - [ ] 编写重试逻辑的单元测试（mock API 响应）

- [ ] Task 6: 前端待分析记录展示 (AC: 5, 6)
  - [ ] 修改 `get_today_records` 返回值包含 analysis_status 字段
  - [ ] 截图画廊中 pending 记录显示黄色边框 + "待分析" 标签
  - [ ] 截图画廊中 failed 记录显示灰色边框 + "分析失败" 标签
  - [ ] 添加"重试分析"按钮（仅对 pending/failed 记录显示）
  - [ ] 编写前端展示的测试

- [ ] Task 7: 手动重试命令 (AC: 6)
  - [ ] 实现 Rust `retry_single_analysis(record_id)` Tauri 命令
  - [ ] 前端调用该命令并更新记录状态
  - [ ] 编写手动重试的端到端测试

## Dev Notes

### 核心设计决策：截图与分析解耦

**当前问题**：`capture_and_store()` 中 `analyze_screen().await?` 失败会导致整个截图流程失败，截图文件虽已保存到磁盘但数据库中无对应记录（孤儿文件）。

**解决方案**：将流程分为两阶段：
1. **阶段一（必须成功）**：截屏 → 保存文件 → 插入 records（不含 AI 分析）
2. **阶段二（可失败）**：AI 分析 → 成功则更新 records content；失败则标记 pending

```rust
// 伪代码 - capture_and_store 重构
let screenshot_path = save_screenshot(&image_base64)?;
let record_id = add_record_pending(timestamp, "auto", "", &screenshot_path)?;

match analyze_screen(&settings, &image_base64).await {
    Ok(analysis) => {
        update_record_analysis(record_id, &analysis.to_json(), "completed")?;
    }
    Err(e) => {
        tracing::warn!("AI analysis failed, queued for retry: {}", e);
        update_record_status(record_id, "pending_analysis")?;
    }
}
```

### 数据库变更

**新增字段**（ALTER TABLE 幂等迁移）：
```sql
ALTER TABLE records ADD COLUMN analysis_status TEXT DEFAULT 'completed';
ALTER TABLE records ADD COLUMN retry_count INTEGER DEFAULT 0;
```

**analysis_status 取值**：
- `completed`：分析成功（默认值，向后兼容已有数据）
- `pending_analysis`：等待 AI 分析
- `analysis_failed`：重试耗尽，分析失败

**新增查询函数**：
```rust
pub fn get_pending_records_sync() -> Result<Vec<Record>, String> {
    // SELECT * FROM records WHERE analysis_status = 'pending_analysis'
    // ORDER BY timestamp ASC
}

pub fn update_record_analysis(id: i64, content: &str, status: &str) -> Result<(), String> {
    // UPDATE records SET content = ?1, analysis_status = ?2 WHERE id = ?3
}

pub fn increment_retry_count(id: i64) -> Result<i32, String> {
    // UPDATE records SET retry_count = retry_count + 1 WHERE id = ?1
    // RETURNING retry_count
}
```

### API 连通性检测

```rust
pub async fn check_api_connectivity() -> Result<bool, String> {
    let settings = memory_storage::get_settings_sync()?;
    let base_url = settings.api_base_url.unwrap_or_default();
    if base_url.is_empty() {
        return Ok(false);
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build().map_err(|e| e.to_string())?;
    // HEAD 请求到 base_url（不消耗 API 配额）
    match client.head(&base_url).send().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
```

### 重试引擎设计

```rust
async fn retry_pending_analyses(settings: &Settings) {
    let pending = match get_pending_records_sync() {
        Ok(records) => records,
        Err(_) => return,
    };
    for record in pending {
        let retry_count = record.retry_count.unwrap_or(0);
        if retry_count >= 3 {
            let _ = update_record_status(record.id, "analysis_failed");
            continue;
        }
        // 指数退避延迟
        let delay_secs = std::cmp::min(30 * 2u64.pow(retry_count as u32), 300);
        tokio::time::sleep(Duration::from_secs(delay_secs)).await;

        // 重新分析（需要读取截图文件重新 base64 编码）
        match retry_analysis_for_record(&record, settings).await {
            Ok(content) => {
                let _ = update_record_analysis(record.id, &content, "completed");
            }
            Err(_) => {
                let _ = increment_retry_count(record.id);
            }
        }
    }
}
```

### 前端离线指示器

```vue
<!-- 在 App.vue 状态栏区域 -->
<div class="flex items-center gap-1 text-xs">
  <span :class="isApiOnline ? 'bg-green-500' : 'bg-red-500'"
        class="w-2 h-2 rounded-full inline-block"></span>
  <span :class="isApiOnline ? 'text-green-400' : 'text-red-400'">
    {{ isApiOnline ? '在线' : '离线' }}
  </span>
</div>
```

### 架构合规要求

- **数据库访问**：使用全局 `DB_CONNECTION` Mutex，不创建新连接
- **参数化查询**：必须使用 `params![]` 宏
- **错误处理**：`Result<T, String>` + `.map_err(|e| e.to_string())`
- **新 Tauri 命令**：注册在 `main.rs` 的 `generate_handler![]`
- **前端样式**：仅 TailwindCSS，自定义主题色 `bg-dark`, `text-primary`
- **后台任务**：使用 `tokio::spawn()`，通过 `AtomicBool` 控制停止
- **时区处理**：使用 `.and_local_timezone(chrono::Local).unwrap().with_timezone(&chrono::Utc)` 转换
- **测试隔离**：使用 `#[serial]` 属性标记访问全局 DB 的测试

### 文件结构要求

**修改文件**：
```
src-tauri/src/
├── auto_perception/mod.rs    # 重构 capture_and_store、添加重试引擎
├── memory_storage/mod.rs     # 新增字段迁移、查询函数、Record 结构更新
├── main.rs                   # 注册 check_api_connectivity、retry_single_analysis 命令

src/
├── App.vue                   # 添加 isApiOnline 状态、离线指示器、定期检测
├── components/
│   └── ScreenshotGallery.vue # pending/failed 记录样式、重试按钮
```

**不需要新增文件**：所有逻辑在现有模块中扩展。

### 测试要求

**Rust 测试重点**：
1. 数据库迁移：新字段默认值正确（已有记录 analysis_status='completed'）
2. `get_pending_records_sync()`：正确筛选 pending 记录
3. `update_record_analysis()`：正确更新 content 和状态
4. `increment_retry_count()`：正确递增并返回新值
5. `capture_and_store` 重构：API 失败时仍创建记录
6. `check_api_connectivity`：超时返回 false，正常返回 true

**前端测试重点**：
1. 离线指示器正确显示在线/离线状态
2. 截图画廊中 pending/failed 记录样式正确
3. 重试按钮仅对 pending/failed 记录显示
4. 点击重试按钮调用正确的 Tauri 命令

**边界测试**：
1. 应用启动时 API 不可用 → 截图仍能保存
2. 分析途中网络断开 → 当前记录标记 pending
3. retry_count 从 0 递增至 3 后标记 failed
4. 网络恢复后 pending 记录自动消化

### 已有可复用的模式

| 模式 | 来源 | 复用方式 |
|------|------|---------|
| Toast 错误通知 + 重试回调 | CORE-004 `src/stores/toast.js` | 重试失败时显示 Toast |
| 错误分类（NETWORK 类型检测） | CORE-004 `src/utils/errors.js` | 区分网络错误和其他错误 |
| ALTER TABLE 幂等迁移 | CORE-003/SMART-002 | `let _ = conn.execute(...)` |
| `#[serial]` 测试隔离 | CORE-005/SMART-003 | 全局 DB 测试必须 serial |
| AtomicBool 后台任务控制 | auto_perception 现有模式 | 重试引擎复用 |
| Settings 字段新增五步同步 | CORE-003 经验 | 迁移→结构体→SELECT→UPDATE→测试 |

## Previous Story Intelligence

### 从 CORE-004 学习的经验

1. **Toast 系统**：`showError(err, retryCallback)` 模式，直接复用
2. **错误分类**：NETWORK 类型检测关键词已定义完整
3. **前端测试模式**：使用 `@vue/test-utils` + `vi.mock('@tauri-apps/api/core')`

### 从 CORE-006 学习的经验

1. **Settings 字段新增清单**：迁移→结构体→SELECT→UPDATE→测试 helper 五处同步
2. **幂等迁移模式**：`let _ = conn.execute("ALTER TABLE ...")` 忽略已存在错误
3. **Rust 测试**：每个 AC 对应多个测试用例

### 从 SMART-002 学习的经验

1. **`max_silent_minutes` 配置模式**：类似的整数配置字段在 settings 表中的存储/读取模式
2. **后台任务定时逻辑**：`tokio::time::interval()` 的使用方式

### 从 SMART-003 学习的经验

1. **布尔字段模式**：`map(|v| if v { 1 } else { 0 })` 存储，`map(|v| v != 0)` 读取
2. **全局 DB 测试必须 `#[serial]`**：避免 flaky test

### 从 CORE 复盘学习的经验

1. **Settings 表字段膨胀问题**：已有 20+ 字段，本次新增 2 个在 records 表（非 settings 表），避免加剧膨胀
2. **全局状态测试隔离**：`AtomicBool` 和 `DB_CONNECTION` 的测试需要 `#[serial]`

### 从 git 近期提交学习

1. 最近修复了 flaky test（`#[serial]` 属性遗漏），所有新的 DB 测试必须加 `#[serial]`
2. v1.8.1 已发布，当前在 main 分支

## Project Structure Notes

### 现有项目结构（与本次变更相关）

```
src-tauri/src/
├── lib.rs                     # AppState, 模块导出
├── main.rs                    # generate_handler![] 命令注册
├── ollama.rs                  # Ollama API 检测和错误格式化
├── auto_perception/
│   └── mod.rs                 # capture_and_store (L765-852), start_auto_capture (L855-964)
├── memory_storage/
│   └── mod.rs                 # Record struct (L31-41), Settings struct (L338-368), add_record (L407-426)
└── synthesis/
    └── mod.rs                 # generate_daily_summary (L249-325)

src/
├── App.vue                    # 主界面，状态管理 (L469-613)
├── utils/errors.js            # ErrorType enum, parseError()
├── stores/toast.js            # showError(err, retryCallback)
└── components/
    └── ScreenshotGallery.vue  # 截图画廊，需添加 pending/failed 样式
```

### 关键代码位置

- **capture_and_store**: `auto_perception/mod.rs:765-852` - 需要重构解耦
- **analyze_screen**: `auto_perception/mod.rs:504-622` - AI API 调用
- **add_record**: `memory_storage/mod.rs:407-426` - 记录插入
- **Record struct**: `memory_storage/mod.rs:31-41` - 需要扩展字段
- **init_database**: `memory_storage/mod.rs` - 需要添加 ALTER TABLE 迁移
- **generate_handler![]**: `main.rs` - 注册新命令

## References

- [Source: epics.md#Epic 1] - CORE-007 离线模式支持定义
- [Source: epics.md#CORE-004 验收条件] - NFR 7.4 可用性要求（离线状态、自动重连、重试机制）
- [Source: architecture.md#4.1 全局状态管理] - DB_CONNECTION Mutex 模式
- [Source: architecture.md#3.1 自动截图流程] - capture → fingerprint → analyze → store 流程
- [Source: architecture.md#5 数据库设计] - records/settings 表结构
- [Source: CORE-004.md] - Toast 系统和错误处理基础设施
- [Source: CORE-006.md] - Settings 字段扩展模式
- [Source: CORE-retrospective.md] - CORE epic 技术债务和经验教训
- [Source: CLAUDE.md] - 项目开发规范、TDD 要求
- [Source: auto_perception/mod.rs:765-852] - 当前 capture_and_store 实现
- [Source: memory_storage/mod.rs:407-426] - 当前 add_record 实现
- [Source: src/utils/errors.js] - NETWORK 错误类型检测
- [Source: src/stores/toast.js] - showError + retry 回调模式

## Dev Agent Record

### Agent Model Used

待开发时填写

### Debug Log References

### Completion Notes List

### File List
