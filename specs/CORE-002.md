# 截图画廊增强规格

## 功能需求
改进 DailyLogger 截图画廊的浏览体验，支持分页加载、视图切换和快速预览，提升用户回顾截图的效率。

## 优化范围
1. 支持网格视图和列表视图切换
2. 支持按日期范围筛选截图
3. 点击缩略图可快速预览大图和 AI 分析内容
4. 分页加载避免一次性加载过多数据
5. 显示截图元信息（时间、AI 分析摘要）

## 不在范围内
- 不修改截图存储逻辑
- 不修改 AI 分析流程
- 不修改数据库 schema

## 验收条件（Given/When/Then）

### AC1 - 视图切换
- Given 用户打开截图画廊
- When 点击视图切换按钮
- Then 可在网格视图（3 列缩略图）和列表视图（带详细信息）之间切换

### AC2 - 日期筛选
- Given 用户打开截图画廊
- When 选择日期范围并点击筛选
- Then 仅显示选定日期范围内的截图记录

### AC3 - 快速预览
- Given 截图画廊中有截图
- When 用户点击任意缩略图
- Then 弹窗显示大图和完整的 AI 分析内容

### AC4 - 分页加载
- Given 截图数量超过 20 条
- When 用户滚动到列表底部
- Then 自动加载下一页 20 条截图

### AC5 - 元信息显示
- Given 截图画廊加载完成
- When 用户查看缩略图
- Then 每张截图显示时间戳和 AI 分析摘要（前 50 字）

## 技术约束
- 只使用 TailwindCSS（无内联样式，无 per-component CSS）
- 前端测试：`npm run test`（Vitest）须通过
- 复用现有 ScreenshotGallery.vue 组件
- 分页使用前端分页（一次性加载全部，前端分页显示）

## 接口定义

### 前端组件
- `ScreenshotGallery.vue` - 主画廊组件
- `ScreenshotModal.vue` - 大图预览弹窗

### 后端 API（已存在，无需修改）
- `get_today_records()` - 获取当日记录
- `get_screenshot(path: String)` - 获取截图文件

## 依赖
- CORE-001: 设置界面优化（已完成）
