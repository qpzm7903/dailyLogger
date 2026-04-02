# 技术架构文档

## DailyLogger 系统架构

---

### 1. 架构概览

DailyLogger 采用 Tauri v2 桌面应用架构，前端 Vue 3 与后端 Rust 通过 IPC 通信。

```
┌─────────────────────────────────────────────────────────────┐
│                     Vue 3 前端 (src/)                        │
│  ┌─────────────┐ ┌──────────────┐ ┌───────────────────────┐ │
│  │ 主界面      │ │ 设置模态框   │ │ 截图画廊              │ │
│  │ App.vue     │ │ SettingsModal│ │ ScreenshotGallery     │ │
│  └─────────────┘ └──────────────┘ └───────────────────────┘ │
│                                                              │
│  invoke() / listen() ← Tauri IPC → generate_handler![]      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                  Rust 后端 (src-tauri/src/)                  │
│  ┌─────────────┐ ┌──────────────┐ ┌───────────────────────┐ │
│  │ auto_       │ │ manual_      │ │ memory_               │ │
│  │ perception  │ │ entry        │ │ storage               │ │
│  └─────────────┘ └──────────────┘ └───────────────────────┘ │
│  ┌─────────────────────┐ ┌───────────────────────────────┐ │
│  │ session_manager     │ │ synthesis (AI 日报生成)         │ │
│  │ (工作时段管理与分析) │ │                                │ │
│  └─────────────────────┘ └───────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      数据持久层                              │
│  ┌─────────────────┐    ┌─────────────────────────────────┐ │
│  │ SQLite (local.db)│    │ 文件系统 (screenshots/, logs/)  │ │
│  │ - records 表     │    │ - 截图 PNG 文件                  │ │
│  │ - settings 表    │    │ - 日志文件                      │ │
│  └─────────────────┘    └─────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

### 2. 模块架构

#### 2.1 前端模块 (src/)

| 组件 | 职责 | 关键功能 |
|-----|------|---------|
| `App.vue` | 主界面容器 | 状态管理、数据加载、视图协调 |
| `SettingsModal.vue` | 设置界面 | API 配置、Obsidian 路径、截图间隔 |
| `QuickNoteModal.vue` | 速记输入 | 快捷键触发、文件读取、内容保存 |
| `ScreenshotModal.vue` | 截图查看 | 大图预览、AI 分析内容展示 |
| `ScreenshotGallery.vue` | 截图画廊 | 网格浏览、分页加载 |
| `DailySummaryViewer.vue` | 日报查看 | Markdown 预览、文件打开 |
| `LogViewer.vue` | 日志查看 | 应用日志浏览、错误诊断 |

**前端技术栈约束** (符合 PRD Section 8):
- Vue 3 Composition API + `<script setup>` 语法
- **TailwindCSS**: 唯一样式方案，无独立 CSS 文件
- 自定义主题色: `bg-dark`, `bg-darker`, `text-primary` (定义在 `tailwind.config.js`)

#### 2.2 后端模块 (src-tauri/src/)

**lib.rs** - 核心库入口
- `AppState`: 全局状态 (单例，Mutex 保护)
- `add()`: 工具函数
- `init_app()`: 应用初始化

**auto_perception/mod.rs** - 自动感知模块（捕获管线）
```rust
// 核心函数
pub fn start_auto_capture()  // 启动后台截图任务
pub fn stop_auto_capture()   // 停止截图
pub fn trigger_capture()     // 手动触发一次截图（不分析）
pub fn take_screenshot()     // 仅截图预览

// 内部流程（v3.0.0: 不再调用 analyze_screen）
capture_screen() → compute_fingerprint() → should_capture()
    → quality_filter() → save_screenshot()
    → detect_or_create_session() → add_record(status: pending)
```

**session_manager/mod.rs** - 工作时段管理模块（分析管线）
```rust
// 核心函数
pub fn detect_or_create_session()    // 检测/创建当前时段
pub fn end_session()                 // 结束时段并触发分析
pub fn analyze_session()             // 批量上下文分析（时段内所有截图 + 上一时段上下文）
pub fn manual_analyze_session()      // 手动触发指定时段分析
pub fn update_screenshot_analysis()  // 更新单张截图分析结果
pub fn update_session_summary()      // 更新时段摘要（用户编辑）
pub fn get_today_sessions()          // 获取今日所有时段
pub fn get_session_screenshots()     // 获取时段内所有截图

// 时段检测逻辑
// 两次截图间隔 > session_gap_minutes(默认30min) → 新时段
// 后台监控任务检测时段结束 → 自动触发 analyze_session()

// 分析流程
// 1. 收集时段内所有截图
// 2. 获取上一时段的 context_for_next
// 3. 构建多图 + 上下文 prompt
// 4. 调用 AI API（批量分析）
// 5. 解析返回：per-screenshot analysis + session summary
// 6. 更新 records.content + sessions.ai_summary
// 7. 生成 context_for_next 供下一时段使用
```

**manual_entry/mod.rs** - 手动输入模块
```rust
pub fn add_quick_note()      // 添加速记
pub fn read_file()           // 读取文件内容
pub fn get_screenshot()      // 获取截图文件
pub fn get_recent_logs()     // 获取最近日志
```

**memory_storage/mod.rs** - 数据存储模块
```rust
pub fn init_database()       // 初始化 SQLite
pub fn add_record()          // 插入记录
pub fn get_today_records()   // 查询今日记录
pub fn get_settings()        // 查询设置
pub fn save_settings()       // 保存设置

// 数据库 Schema
// records: id, timestamp, source_type, content, screenshot_path
// settings: 单行配置表
```

**synthesis/mod.rs** - 合成模块
```rust
pub fn generate_daily_summary()  // AI 生成日报
// 流程：
// 1. get_today_records() 获取全天记录
// 2. 构建 AI Prompt (包含所有记录 + 截图)
// 3. 调用 OpenAI API 生成 Markdown
// 4. 保存到 Obsidian 路径
// 5. 更新 settings.last_summary_path
```

---

### 3. 数据流

#### 3.1 捕获管线（不调用 AI）

```
用户点击"启动"或设置 auto_capture_enabled=1
         ↓
start_auto_capture() → tokio::spawn 后台任务
         ↓
    ┌──────────────────────────────────────┐
    │  循环执行 (每 N 分钟)                  │
    │  1. capture_screen()                 │
    │     → Base64 编码 PNG                  │
    │  2. compute_fingerprint()            │
    │     → 64x64 灰度缩略图                  │
    │  3. should_capture()                 │
    │     → 对比上次指纹，变化率 < 阈值？跳过 │
    │  4. quality_filter()                 │
    │     → 低质量截图？跳过                  │
    │  5. save_screenshot()                │
    │     → 写入 screenshots/               │
    │  6. detect_or_create_session()       │
    │     → 距上一截图 > 30min？创建新时段   │
    │  7. add_record(status: pending)      │
    │     → 存入 SQLite（不含 AI 分析）       │
    └──────────────────────────────────────┘
         ↓
用户界面自动刷新 (每 30 秒轮询, 显示待分析状态)
```

#### 3.2 分析管线（AI 调用）

```
触发方式:
  A. 时段自动结束 (检测到 > 30min 间隔)
  B. 用户手动触发
  C. 日报生成时自动分析未处理时段
         ↓
analyze_session(session_id)
         ↓
1. 收集该时段所有截图 (records where session_id = ?)
2. 获取上一时段的 context_for_next
3. 构建批量分析 Prompt (多图 + 文本上下文)
4. 发送给 AI API
5. 解析返回:
   - 每张截图的独立分析 → records.content
   - 时段摘要 → sessions.ai_summary
   - 上下文 → sessions.context_for_next
6. 更新 records.analysis_status → 'analyzed'
         ↓
用户可编辑:
  截图级 → records.user_notes (优先于 content)
  时段级 → sessions.user_summary (优先于 ai_summary)
```

#### 3.3 日报生成流程

```
用户点击"生成日报"
         ↓
generate_daily_summary()
         ↓
get_today_sessions() → Vec<Session>
  (未分析的时段自动触发 analyze_session)
         ↓
构建 Prompt (按时段组织):
"以下是用户今日各工作时段的分析结果：
- 时段 1 (09:00-11:30): {user_summary 或 ai_summary}
- 时段 2 (13:00-15:00): {user_summary 或 ai_summary}
请生成结构化日报..."
         ↓
调用 OpenAI API (chat/completions)
         ↓
写入 Obsidian 路径/YYYY-MM-DD.md
         ↓
返回文件路径给前端
```

---

### 4. 关键设计决策

#### 4.1 全局状态管理

**问题**: Rust 测试并行执行，共享 DB_CONNECTION 导致竞争

**解决方案**:
```rust
// lib.rs
pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

// memory_storage/mod.rs
static DB_CONNECTION: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

// 使用模式
let db = DB_CONNECTION.lock().map_err(|e| ...)?;
let conn = db.as_ref().ok_or("Not initialized")?;
```

#### 4.2 截图去重优化

**问题**: 屏幕无变化时重复截图浪费资源

**解决方案**: 指纹对比 + 时间阈值
```rust
// 64x64 灰度 = 4096 bytes
fn compute_fingerprint(image_base64: &str) -> Vec<u8>

// 变化率计算
fn calc_change_rate(a: &[u8], b: &[u8]) -> f64
// NOISE_TOLERANCE = 10 (像素差)
// 超过阈值才计入变化

// 强制捕获
const DEFAULT_MAX_SILENT_MINUTES: u64 = 30
// 即使无变化，30 分钟后也必须捕获一次
```

#### 4.3 时区处理

**问题**: `.and_utc()` 在 UTC+8 时区会丢失本地凌晨记录

**正确方案**:
```rust
let today_start = chrono::Local::now()
    .date_naive()
    .and_hms_opt(0, 0, 0).unwrap()
    .and_local_timezone(chrono::Local)
    .unwrap()
    .with_timezone(&chrono::Utc)
    .to_rfc3339();
```

#### 4.4 跨平台截图

```rust
// Windows: Windows Graphics Capture API
#[cfg(target_os = "windows")]
fn capture_screen() -> Result<String, String> {
    use windows_capture::{...}
}

// macOS/Linux: xcap
#[cfg(not(target_os = "windows"))]
fn capture_screen() -> Result<String, String> {
    let monitors = xcap::Monitor::all()?;
    monitors[0].capture_image()
}
```

---

### 5. 数据库设计

#### 5.1 records 表

```sql
CREATE TABLE records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,          -- RFC3339 UTC
    source_type TEXT NOT NULL,        -- 'auto' | 'manual'
    content TEXT NOT NULL,            -- AI 分析结果 JSON 或纯文本
    screenshot_path TEXT,             -- 相对路径或 NULL
    monitor_info TEXT,                -- JSON: MonitorInfo
    tags TEXT,                        -- JSON: Vec<String>
    user_notes TEXT,                  -- 用户自写分析/备注（优先于 content）
    session_id INTEGER,               -- 所属工作时段 (v3.0.0)
    analysis_status TEXT DEFAULT 'pending'  -- pending | analyzed | user_edited (v3.0.0)
);

-- 索引优化
CREATE INDEX idx_timestamp ON records(timestamp DESC);
CREATE INDEX idx_source_type ON records(source_type);
CREATE INDEX idx_session_id ON records(session_id);
```

#### 5.1.1 sessions 表 (v3.0.0 新增)

```sql
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,                    -- YYYY-MM-DD
    start_time TEXT NOT NULL,              -- RFC3339
    end_time TEXT,                         -- RFC3339, NULL = ongoing
    ai_summary TEXT,                       -- AI 生成的时段摘要
    user_summary TEXT,                     -- 用户自写的时段摘要（优先于 ai_summary）
    context_for_next TEXT,                 -- 传递给下一时段分析的上下文
    status TEXT DEFAULT 'active'           -- active | ended | analyzed
);

CREATE INDEX idx_sessions_date ON sessions(date);
```

**优先级规则**:
- UI 展示和日报生成时，`user_notes` / `user_summary` 优先于 AI 结果
- 如果用户编辑了内容，`analysis_status` 更新为 `user_edited`

#### 5.2 settings 表

```sql
CREATE TABLE settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- 强制单行

    -- AI 配置
    api_base_url TEXT,
    api_key TEXT,
    model_name TEXT,

    -- 捕获配置
    screenshot_interval INTEGER DEFAULT 5,
    change_threshold INTEGER DEFAULT 3,      -- 变化率阈值 (%)
    max_silent_minutes INTEGER DEFAULT 30,   -- 强制捕获时间

    -- 分析配置
    analysis_prompt TEXT,
    summary_model_name TEXT,
    summary_prompt TEXT,

    -- 输出配置
    summary_time TEXT DEFAULT '18:00',
    obsidian_path TEXT,
    auto_capture_enabled INTEGER DEFAULT 0,
    last_summary_path TEXT,

    -- AI-006: 自定义 API Headers
    custom_headers TEXT DEFAULT '[]'   -- JSON: Vec<CustomHeader>
);
```

---

### 6. API 端点 (Tauri Commands)

| 命令 | 模块 | 描述 |
|-----|------|------|
| `start_auto_capture` | auto_perception | 启动后台截图循环（不分析） |
| `stop_auto_capture` | auto_perception | 停止截图 |
| `trigger_capture` | auto_perception | 手动触发一次截图（不分析） |
| `take_screenshot` | auto_perception | 仅截图预览 |
| `add_quick_note` | manual_entry | 添加速记 |
| `read_file` | manual_entry | 读取文件内容 |
| `get_screenshot` | manual_entry | 获取截图文件 |
| `get_recent_logs` | manual_entry | 获取最近日志 |
| `get_today_records` | memory_storage | 查询今日记录 |
| `get_settings` | memory_storage | 查询设置 |
| `save_settings` | memory_storage | 保存设置 |
| `generate_daily_summary` | synthesis | 生成日报（基于时段分析） |
| `get_today_sessions` | session_manager | 获取今日所有工作时段 |
| `get_session_screenshots` | session_manager | 获取时段内所有截图 |
| `analyze_session` | session_manager | 触发指定时段的批量分析 |
| `update_record_user_notes` | session_manager | 更新截图的用户备注 |
| `update_session_summary` | session_manager | 更新时段的用户摘要 |

---

### 7. 文件系统

```
~/.local/share/DailyLogger/
├── data/
│   └── local.db            # SQLite 数据库
├── screenshots/
│   ├── screenshot_20260313_090000.png
│   └── ...
├── logs/
│   └── daily-logger.log    # 滚动日志 (Rotation::NEVER)
└── obsidian/
    └── 工作日报-2026-03-13.md
```

---

### 8. 测试策略

#### 8.1 Rust 测试

```rust
// 测试分类
#[cfg(test)]
mod tests {
    // 1. 单元测试：纯函数
    #[test]
    fn calc_change_rate_identical_images_returns_zero()

    // 2. 边界测试：时区、时间戳
    #[test]
    fn finds_record_saved_near_local_midnight()

    // 3. 端到端测试：CRUD 流程
    #[test]
    fn add_record_then_query_returns_it()

    // 4. 集成测试：AI API 调用 (mock)
}
```

#### 8.2 前端测试

```typescript
// Vitest + @vue/test-utils
describe('App.vue', () => {
  it('loads today records on mount')
  it('toggles auto capture state')
  it('generates summary and shows path')
})
```

---

### 9. 扩展性设计

#### 9.1 插件化 Prompt

```rust
// settings 表支持自定义 Prompt
pub struct Settings {
    pub analysis_prompt: Option<String>,   // 截图分析 Prompt
    pub summary_prompt: Option<String>,    // 日报生成 Prompt
}
```

#### 9.2 多模型支持

```rust
pub struct Settings {
    pub model_name: Option<String>,         // 分析用模型
    pub summary_model_name: Option<String>, // 总结用模型
}
```

#### 9.3 智能去重

```rust
pub struct Settings {
    pub change_threshold: Option<i32>,     // 变化率阈值 (%)
    pub max_silent_minutes: Option<i32>,   // 强制捕获时间
}
```

#### 9.4 自定义 API Headers (AI-006)

支持 OpenRouter、Azure OpenAI、Claude 等 API 服务所需的自定义请求头。

```rust
// 数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHeader {
    pub key: String,        // Header 名称
    pub value: String,      // Header 值
    pub sensitive: bool,    // 是否加密存储
}

// Settings 字段
pub struct Settings {
    pub custom_headers: Option<String>,  // JSON: Vec<CustomHeader>
}

// API 调用时应用 Headers
fn call_llm_api(config: &ApiConfig, ...) {
    // 检查自定义 Headers 是否包含 Authorization 或 api-key
    let has_custom_auth = config.custom_headers.iter().any(|h| {
        h.key.to_lowercase() == "authorization" || h.key.to_lowercase() == "api-key"
    });

    // 仅在没有自定义认证头时添加默认 Authorization
    if !config.api_key.is_empty() && !has_custom_auth {
        request = request.header("Authorization", format!("Bearer {}", config.api_key));
    }

    // 应用自定义 Headers
    for header in &config.custom_headers {
        request = request.header(&header.key, &header.value);
    }
}
```

**预设模板**:
- OpenRouter: `HTTP-Referer`, `X-Title`
- Azure OpenAI: `api-key` (替代 Authorization)
- Claude API: `anthropic-version`

---

### 10. 性能优化

| 优化点 | 策略 | 效果 |
|-------|------|------|
| 截图去重 | 指纹对比 + 阈值 | 减少 70% AI 调用 |
| 缩略图指纹 | 64x64 灰度 = 4KB | 内存占用极低 |
| 数据库索引 | timestamp DESC | 查询 <10ms |
| 滚动日志 | Rotation::NEVER | 单文件易管理 |
| 前端轮询 | 30 秒间隔 | 降低 IPC 调用频率 |

---

### 11. 安全设计

| 风险 | 缓解措施 |
|-----|---------|
| API Key 泄露 | AES-256 加密存储 (CORE-006)，使用 aes-gcm crate，密钥基于机器特征生成，mask 后 logging |
| 截图隐私 | 仅本地处理，可选关闭自动捕获 |
| 数据丢失 | 单文件数据库易备份 |
| 网络攻击 | 无对外服务端口，纯客户端架构 |

**API Key 加密实现 (CORE-006)**:
- 加密算法: AES-256-GCM
- 密钥生成: 基于机器特征 (设备 ID)
- 密钥存储: 应用数据目录 `.key` 文件，权限 600
- 迁移策略: 自动检测并加密明文 Key
- 内存安全: API 调用后及时清除敏感字符串

---

### 12. 报告输出模块

**synthesis/mod.rs + services/report_service.rs** - 本地报告生成与输出
```rust
// 核心函数
pub async fn generate_daily_summary()     // 生成日报 Markdown
pub async fn generate_weekly_report()     // 生成周报 Markdown
pub async fn generate_monthly_report()    // 生成月报 Markdown
fn write_report_to_obsidian()             // 写入配置的 Obsidian Vault
```

**输出设计要点**:

| 能力 | 说明 |
|------|------|
| Markdown 生成 | 日报、周报、月报统一走本地 Markdown 输出链路 |
| 路径解析 | 基于设置解析 Vault 与目标目录 |
| 失败隔离 | 写出失败记录日志，不阻塞主流程 |
| 本地优先 | 不依赖外部平台 API，降低维护复杂度 |

**实现细节**:
- 输出文件名由日期范围和模板统一生成
- Obsidian 写出沿用本地文件系统能力，无网络依赖
- 错误处理使用 `tracing::warn!` 记录日志
- 通过设置层控制是否启用输出及目标路径

---

**文档更新**: 2026-03-22
**版本**: 3.0.0 (分析管线重设计: 工作时段感知分析; 移除非核心第三方集成)
