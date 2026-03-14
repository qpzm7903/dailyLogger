# Dev Log

Key technical decisions, problems encountered, and conventions from story implementations.

---

## AI-003 - 2026-03-14

### 技术决策

1. **日报 Prompt 模板库**：预设模板定义在前端静态数组中，默认模板内容运行时从后端获取。理由：保持默认 Prompt 的单一事实来源在后端代码中。

2. **导入验证策略**：导入 JSON 模板时验证 `{records}` 占位符是否存在。理由：确保导入的模板可以正常使用，避免运行时错误。

3. **模板导出格式**：采用标准 JSON 格式，包含 version、name、description、content、createdAt 字段。理由：便于用户备份和分享模板配置。

### 遇到问题

开发过程顺利，所有测试通过。TDD 流程先写测试后实现，后端 41 测试通过，前端 92 测试通过。

### 后续约定

- **模板占位符**：日报 Prompt 必须包含 `{records}` 占位符
- **导入错误提示**：JSON 解析失败、缺少 content 字段、缺少占位符分别提供具体错误信息

---

## AI-002 - 2026-03-14

### 技术决策

1. **默认 Prompt 查询**：使用 `get_default_analysis_prompt` Tauri command 返回静态常量，而非存储在 DB。理由：保持默认 Prompt 的单一事实来源在代码中，便于版本升级时自动更新。

2. **重置策略**：重置时清空 `analysis_prompt` 字段而非填入默认值。理由：后端在字段为空时自动使用 DEFAULT_ANALYSIS_PROMPT，避免前后端默认值不一致。

3. **Modal 展示**：用独立 Modal 展示默认 Prompt 而非 Tooltip。理由：Prompt 内容较长（~500 字符），Modal 提供更好的阅读体验和滚动支持。

### 遇到问题

无重大问题。开发过程顺利，TDD 流程先写测试后实现，全部 92 测试通过。

### 后续约定

- **常量暴露模式**：需向前端暴露只读常量时，创建无参数 Tauri command 返回静态值
- **重置模式**：字段重置优先清空而非填默认值，依赖后端 fallback 逻辑
- **命令注册**：新增 Tauri command 必须在 `main.rs` 的 `generate_handler![]` 中注册

---

## CORE-002 - 2026-03-14

### 技术决策

1. **视图状态管理**：使用简单 `ref('grid')` 管理视图状态，默认网格视图。理由：无需复杂状态管理，单一组件内状态足够。

2. **响应式网格布局**：采用 `grid-cols-1 md:grid-cols-2 lg:grid-cols-3` 实现三列响应式。理由：在不同屏幕尺寸下自适应，符合 Tailwind 最佳实践。

3. **按钮高亮模式**：活动按钮 `bg-primary text-white`，非活动 `bg-darker text-gray-400 hover:text-white`。理由：提供清晰的视觉反馈，保持与现有主题一致。

4. **列表视图时间格式**：新增 `formatTimeShort` 函数返回 HH:MM:SS 格式。理由：列表视图空间有限，短格式更紧凑。

5. **日期范围边界处理**：start_date 取 00:00:00，end_date 取 23:59:59，确保同一天记录全包含。理由：用户期望日期筛选包含边界值，符合直觉。

6. **xcap 依赖重构**：将 xcap 从条件依赖改为 optional 依赖，在 feature 中显式声明。理由：修复 `screenshot` feature 编译问题，依赖声明更清晰。

7. **缩略图加载复用**：抽取 `loadThumbnails` 函数供筛选和默认加载共用。理由：避免代码重复，统一缩略图加载逻辑。

### 遇到问题

**依赖配置问题**：xcap 在 `screenshot` feature 下编译失败，原配置 `[target.'cfg(all(not(target_os = "windows"), feature = "screenshot"))'.dependencies]` 无法正确解析 feature 条件。解决：改为 optional 依赖 + feature 显式引用。

### 后续约定

- **切换按钮组模式**：活动按钮 `bg-primary text-white`，非活动 `bg-darker text-gray-400 hover:text-white`
- **响应式三列**：`grid-cols-1 md:grid-cols-2 lg:grid-cols-3`
- **列表分隔线**：使用 `divide-y divide-gray-700` 实现行分隔
- **测试分组**：按 AC 分组测试用例（如 `AC1 - View Toggle`）提高可读性
- **日期 API 格式**：前端传 `YYYY-MM-DD` 字符串，后端解析为本地时间边界再转 UTC
- **日期筛选测试**：覆盖边界值（00:00:00, 23:59:59）和跨日场景
- **测试隔离**：用 `.iter().any()` 定位记录，不依赖记录顺序或全局数量

### Task 3 技术决策

1. **快速预览复用模式**：直接复用现有 ScreenshotModal.vue 组件，通过 props 传递完整的 record 对象。理由：避免重复代码，保持组件职责单一。

2. **异步测试等待策略**：使用 `waitFor` 辅助函数等待 VM 状态更新而非固定次数的 nextTick。理由：测试更稳定，不依赖具体的异步操作数量。

3. **测试分组策略**：按 AC 分组测试用例（AC1 - View Toggle, AC3 - Quick Preview Modal）。理由：提高测试可读性，便于定位问题。

### Task 3 后续约定

- **异步测试等待**：使用 `waitFor(() => wrapper.vm.xxx)` 等待状态更新
- **预览模态框测试**：验证 record 传递、路径正确性、内容完整性

### Task 3 测试实现 - 2026-03-14

### 技术决策

1. **waitFor 辅助函数**：封装异步等待逻辑，条件检查 + 超时机制。理由：替代固定 nextTick 次数，测试更稳定可靠。

2. **Modal 测试策略**：通过 `findComponent({ name: 'ScreenshotModal' })` 定位子组件，验证 props 和事件。理由：直接访问组件实例，断言更精确。

### 遇到问题

原测试使用多个 nextTick 等待异步操作，在 CI 环境偶尔超时。解决：引入 waitFor 辅助函数，基于条件轮询而非固定次数。

### 后续约定

- **异步测试模式**：`waitFor(() => condition, timeout)` 替代多次 nextTick
- **Modal 测试清单**：1) 验证组件存在 2) 验证 props 传递 3) 验证事件触发 4) 验证状态重置