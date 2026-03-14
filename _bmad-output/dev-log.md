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