# 设置界面优化规格

## 功能需求
改进 DailyLogger 设置界面的用户体验，优化表单布局、验证反馈和配置项分组，提升用户配置效率。

## 优化范围
1. 配置项分组显示（AI 配置、捕获配置、输出配置）
2. 表单输入验证和实时反馈
3. API 连接测试按钮
4. 优化 Obsidian 路径选择器交互
5. 保存成功/失败状态提示

## 不在范围内
- 不新增配置项
- 不修改 Rust 后端逻辑
- 不修改数据库 schema

## 验收条件（Given/When/Then）

### AC1 - 配置分组
- Given 用户打开设置界面
- When 界面加载完成
- Then 配置项按逻辑分组显示（AI 配置、捕获配置、输出配置），每组有清晰的标题

### AC2 - 输入验证
- Given 用户在 API URL 字段输入内容
- When 输入非 URL 格式
- Then 显示验证错误提示

### AC3 - 连接测试
- Given 用户已配置 API URL 和 Key
- When 点击"测试连接"按钮
- Then 发送测试请求并显示成功/失败结果

### AC4 - 路径选择
- Given 用户点击 Obsidian 路径输入框旁的"浏览"按钮
- When 选择目录后
- Then 路径自动填入输入框

### AC5 - 保存反馈
- Given 用户修改设置并点击保存
- When 保存操作完成
- Then 显示成功或失败的 Toast 提示

## 技术约束
- 只使用 TailwindCSS（无内联样式，无 per-component CSS）
- 前端测试：`npm run test`（Vitest）须通过
- 复用现有 SettingsModal.vue 组件

## 接口定义

### 前端组件
- `SettingsModal.vue` - 设置模态框组件（优化）

### 后端 API（已存在）
- `get_settings()` - 获取当前设置
- `save_settings(settings: Settings)` - 保存设置

## 依赖
- 无前置依赖