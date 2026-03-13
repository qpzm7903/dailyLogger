# UI 交互体验优化规格

## 功能需求
改善 DailyLogger 桌面应用的界面交互体验，提升视觉一致性和操作效率。

## 优化范围
1. 统一各组件的颜色主题，确保与 tailwind.config.js 定义的自定义色系（bg-dark, bg-darker, text-primary）一致
2. 改善按钮点击反馈（hover/active 状态）
3. 优化截图列表的滚动体验（长列表）
4. 改善 SettingsModal 表单的输入体验（placeholder 文本、label 间距）

## 不在范围内
- 不新增功能
- 不修改 Rust 后端逻辑
- 不修改数据库 schema

## 验收条件（Given/When/Then）

### AC1 - 颜色一致性
- Given 应用启动
- When 用户打开主界面
- Then 所有背景色使用 bg-dark 或 bg-darker，所有文字使用 text-primary 或 tailwind 默认颜色

### AC2 - 按钮交互反馈
- Given 用户将鼠标移到按钮上
- When hover 发生
- Then 按钮有明显的视觉反馈（颜色/透明度变化）

### AC3 - 设置表单可用性
- Given 用户打开设置界面
- When 查看各输入字段
- Then 每个字段有清晰的 label 和 placeholder 说明

## 技术约束
- 只使用 TailwindCSS（无内联样式，无 per-component CSS）
- 前端测试：`npm run test`（Vitest）须通过
- 无需新增 Vue 组件，只修改现有组件
