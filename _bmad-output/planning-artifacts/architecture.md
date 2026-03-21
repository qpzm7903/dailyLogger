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
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ synthesis (AI 日报生成)                                  │ │
│  └─────────────────────────────────────────────────────────┘ │
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

**auto_perception/mod.rs** - 自动感知模块
```rust
// 核心函数
pub fn start_auto_capture()  // 启动后台截图任务
pub fn stop_auto_capture()   // 停止截图
pub fn trigger_capture()     // 手动触发一次完整捕获
pub fn take_screenshot()     // 仅截图预览 (不调用 AI)

// 内部流程
capture_screen() → compute_fingerprint() → should_capture()
    → save_screenshot() → analyze_screen() → add_record()
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

#### 3.1 自动截图流程

```
用户点击"启动"或设置 auto_capture_enabled=1
         ↓
start_auto_capture() → tokio::spawn 后台任务
         ↓
    ┌────────────────────────────┐
    │  循环执行 (每 N 分钟)        │
    │  1. capture_screen()       │
    │     → Base64 编码 PNG        │
    │  2. compute_fingerprint()  │
    │     → 64x64 灰度缩略图        │
    │  3. should_capture()       │
    │     → 对比上次指纹           │
    │     → 变化率 < 阈值？跳过    │
    │  4. save_screenshot()      │
    │     → 写入 screenshots/     │
    │  5. analyze_screen()       │
    │     → 调用 Vision API       │
    │  6. add_record()           │
    │     → 存入 SQLite           │
    └────────────────────────────┘
         ↓
用户界面自动刷新 (每 30 秒轮询)
```

#### 3.2 日报生成流程

```
用户点击"生成日报"
         ↓
generate_daily_summary()
         ↓
get_today_records() → Vec<Record>
         ↓
构建 Prompt:
"请根据以下工作记录生成日报：
- 09:00 自动：正在编写 Rust 后端代码...
- 09:15 手动：需要实现数据库连接..."
         ↓
调用 OpenAI API (chat/completions)
         ↓
解析 Markdown 响应
         ↓
写入 Obsidian 路径/YYYY-MM-DD.md
         ↓
更新 settings.last_summary_path
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
    timestamp TEXT NOT NULL,      -- RFC3339 UTC
    source_type TEXT NOT NULL,    -- 'auto' | 'manual'
    content TEXT NOT NULL,        -- JSON 或纯文本
    screenshot_path TEXT          -- 相对路径或 NULL
);

-- 索引优化
CREATE INDEX idx_timestamp ON records(timestamp DESC);
CREATE INDEX idx_source_type ON records(source_type);
```

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
| `start_auto_capture` | auto_perception | 启动后台截图循环 |
| `stop_auto_capture` | auto_perception | 停止截图 |
| `trigger_capture` | auto_perception | 手动触发一次完整捕获 |
| `take_screenshot` | auto_perception | 仅截图预览 |
| `add_quick_note` | manual_entry | 添加速记 |
| `read_file` | manual_entry | 读取文件内容 |
| `get_screenshot` | manual_entry | 获取截图文件 |
| `get_recent_logs` | manual_entry | 获取最近日志 |
| `get_today_records` | memory_storage | 查询今日记录 |
| `get_settings` | memory_storage | 查询设置 |
| `save_settings` | memory_storage | 保存设置 |
| `generate_daily_summary` | synthesis | 生成日报 |

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

### 12. Notion 集成模块

**notion.rs** - Notion API 集成
```rust
// 核心函数
pub async fn write_report_to_notion()  // 将报告写入 Notion 数据库
pub fn test_notion_connection()        // 测试连接
pub fn is_notion_configured()          // 检查配置状态

// Markdown 转换
pub fn markdown_to_notion_blocks()     // Markdown → Notion Blocks
fn chunk_blocks()                      // 分块处理（100 blocks/请求）
async fn get_title_property_name()     // 自动检测标题属性名
```

**Markdown 到 Notion Block 转换**:

| Markdown | Notion Block Type |
|----------|------------------|
| `# H1` | `heading_1` |
| `## H2` | `heading_2` |
| `### H3` | `heading_3` |
| `- item` | `bulleted_list_item` |
| `1. item` | `numbered_list_item` |
| ``` code ``` | `code` |
| `> quote` | `quote` |
| `**bold**` | `annotations.bold = true` |
| `*italic*` | `annotations.italic = true` |
| `` `code` `` | `annotations.code = true` |

**实现细节**:
- 使用 `pulldown-cmark` crate 解析 Markdown
- `RichTextBuilder` 模式处理行内格式（bold/italic/code）
- 分块上传：每批最多 100 blocks（Notion API 限制）
- 自动检测数据库标题属性名（支持 Name/Title/标题/名称）
- 错误处理使用 `tracing::warn!` 记录日志

**API 限制处理**:
- 单次 `append_block_children` 请求最多 100 blocks
- Rich text 单个元素最多 2000 字符
- 自动截断超长内容

---

**文档更新**: 2026-03-21
**版本**: 2.1.0 (新增 INT-001 Notion 导出完善)
