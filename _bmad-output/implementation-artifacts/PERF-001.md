# Story 10.1: AI 配置完善（代理支持）

Status: ready-for-dev

## Story

As a DailyLogger user,
I want to configure a proxy for AI API requests,
so that I can use the application behind a corporate firewall or when traveling in regions with restricted network access.

**来源**: Issue #76 - AI的base url 以及test model的配置不全，怀疑代理的问题，最好把代理一起页也放在配置里面，让用户自己决定是否用代理

## Background

当前 BasicSettings.vue 中只有 API Base URL 和 API Key 配置，缺少代理配置。用户在使用代理访问 AI API 时无法指定代理设置。

**Issue #76 背景**：
- 用户反馈 AI 配置不完整，怀疑是代理问题
- 当前没有代理配置选项，用户无法指定使用代理
- 需要让用户自己决定是否使用代理

**相关现有代码**：
- `BasicSettings.vue`：现有 AI 基本配置（API Base URL、API Key、Test Connection）
- `synthesis/mod.rs`：`call_llm_api()` 使用 `create_http_client()` 创建 HTTP 客户端
- `lib.rs`：`create_http_client()` 目前只有 `no_proxy()` 逻辑，没有显式代理配置
- `AI-006`（custom_headers）已支持自定义 API Headers，代理配置应采用类似模式

**Epic 10 定位**：
```
Epic 10: 体验极致化
├── PERF-001: AI 配置完善（代理支持） ← 当前
├── PERF-002: 新用户引导
├── PERF-003: 性能优化 - 截图加载
├── PERF-004: 性能优化 - 数据库查询
├── PERF-005: 多语言支持 (i18n)
└── PERF-006: 浅色主题支持
```

## Acceptance Criteria

1. **代理配置 UI**
   - Given 用户打开设置 → AI 配置面板
   - When 查看 API 配置区域
   - Then 显示"启用代理"开关 + 代理地址输入框 + 代理端口输入框 + 用户名/密码（可选）
   - And 默认折叠（不占用过多空间）

2. **代理配置持久化**
   - Given 用户配置了代理并保存设置
   - When 重启应用
   - Then 代理配置正确恢复
   - And 数据库 schema 正确存储 `proxy_enabled`, `proxy_host`, `proxy_port`, `proxy_username`, `proxy_password`

3. **测试连接使用代理**
   - Given 用户配置了代理
   - When 点击"测试连接"按钮
   - Then 使用配置的代理发送测试请求
   - And 显示正确的成功/失败结果

4. **AI 分析使用代理**
   - Given 用户配置了代理并启用了代理
   - When 应用进行 AI 分析或日报生成
   - Then 所有 AI API 请求通过指定代理发送
   - And `create_http_client()` 使用代理配置

5. **代理认证支持**
   - Given 用户配置了需要认证的代理
   - When 发送 AI API 请求
   - Then 使用 Basic 认证发送请求

6. **Test Model 字段完善**
   - Given 用户在设置中配置了 Test Model
   - When 测试连接时
   - Then 使用配置的 Test Model 进行验证（而非分析 Model）

## Tasks / Subtasks

- [ ] Task 1: 数据库 schema 添加代理配置字段 (AC: #2)
  - [ ] 在 `schema.rs` 添加 `proxy_enabled`, `proxy_host`, `proxy_port`, `proxy_username`, `proxy_password` 字段
  - [ ] 在 `memory_storage/mod.rs` 的 `Settings` 结构体添加对应字段
  - [ ] 运行数据库迁移（ALTER TABLE）

- [ ] Task 2: 前端代理配置 UI (AC: #1)
  - [ ] 在 `BasicSettings.vue` API Configuration 区域添加代理配置面板
  - [ ] 代理配置默认折叠，点击展开
  - [ ] 包含：启用开关、代理地址、端口、用户名（可选）、密码（可选）
  - [ ] 响应式验证：端口仅接受数字

- [ ] Task 3: 后端代理 HTTP Client 支持 (AC: #3, #4)
  - [ ] 修改 `lib.rs` 的 `create_http_client()` 函数，支持显式代理配置
  - [ ] 修改 `synthesis/mod.rs` 的 `ApiConfig` 和 `call_llm_api()`，传入代理配置
  - [ ] 实现 Basic 认证代理支持
  - [ ] 确保所有 AI API 调用都使用代理（session_manager、ollama 等）

- [ ] Task 4: 测试连接使用代理 (AC: #3)
  - [ ] 修改 `test_api_connection_with_ollama` 命令，传入代理配置
  - [ ] 前端测试连接时传递代理参数

- [ ] Task 5: Test Model 字段完善 (AC: #6)
  - [ ] 在 settings 表添加 `test_model_name` 字段（可选）
  - [ ] 前端 UI 添加 Test Model 输入框（位于 Base URL 和 API Key 下方）
  - [ ] 测试连接时优先使用 `test_model_name`（若配置）

- [ ] Task 6: 集成测试 (AC: all)
  - [ ] 手动测试：启用代理 → 测试连接 → 确认成功
  - [ ] 手动测试：不启用代理 → 测试连接 → 确认直连成功
  - [ ] 手动测试：认证代理 → 测试连接 → 确认认证成功

## Dev Notes

### 关键架构约束

1. **前端技术栈**：Vue 3 Composition API + `<script setup>`，TailwindCSS（无独立 CSS 文件）
2. **后端技术栈**：Rust + Tauri v2，`reqwest` 用于 HTTP 请求
3. **代理实现**：`reqwest` 支持通过 `ClientBuilder::proxy()` 设置代理

### 文件树组件（需修改）

```
src/
├── components/settings/
│   └── BasicSettings.vue          # 添加代理配置 UI
src-tauri/src/
├── lib.rs                         # 修改 create_http_client() 支持显式代理
├── memory_storage/
│   ├── mod.rs                    # Settings 结构体添加代理字段
│   └── schema.rs                # 添加代理字段的 ALTER TABLE
├── synthesis/mod.rs              # call_llm_api() 传入代理配置
├── session_manager/mod.rs         # 分析管线使用代理
└── ollama.rs                     # Ollama API 调用使用代理
```

### 数据库 Schema 变更

```sql
ALTER TABLE settings ADD COLUMN proxy_enabled INTEGER DEFAULT 0;
ALTER TABLE settings ADD COLUMN proxy_host TEXT;
ALTER TABLE settings ADD COLUMN proxy_port INTEGER DEFAULT 8080;
ALTER TABLE settings ADD COLUMN proxy_username TEXT;
ALTER TABLE settings ADD COLUMN proxy_password TEXT;
ALTER TABLE settings ADD COLUMN test_model_name TEXT;
```

### reqwest 代理配置示例

```rust
use reqwest::Proxy;

let proxy = Proxy::https("http://proxy.example.com:8080")?
    .basic_auth("user", "password");
let client = Client::builder()
    .proxy(proxy)
    .timeout(Duration::from_secs(120))
    .build()?;
```

### API 配置传递链

```
Settings (DB)
  ↓ load_api_config()
ApiConfig { proxy_enabled, proxy_host, ... }
  ↓ call_llm_api(config)
HTTP Client with proxy
  ↓
reqwest request
```

## Testing Requirements

1. **单元测试**：
   - `lib.rs`: 测试 `create_http_client()` 代理模式和无代理模式
   - `memory_storage`: 测试代理配置序列化和反序列化

2. **集成测试**：
   - 端到端测试代理配置流程
   - 验证所有 AI API 调用（截图分析、日报生成）都使用代理

3. **测试覆盖**：
   - 无代理直连
   - HTTP 代理（无认证）
   - HTTP 代理（Basic 认证）
   - 代理启用/禁用切换

## References

- [Source: src-tauri/src/lib.rs#114] - `create_http_client()` 函数
- [Source: src-tauri/src/synthesis/mod.rs#64] - `call_llm_api()` 函数
- [Source: src/components/settings/BasicSettings.vue] - 现有 AI 配置 UI
- [Source: src-tauri/src/memory_storage/schema.rs] - 数据库 Schema 定义
- [Issue #76](https://github.com/qpzm7903/dailylogger/issues/76) - AI 配置问题反馈

## Dev Agent Record

### Agent Model Used

claude-opus-4-6

### Debug Log References

### Completion Notes List

### File List
