# 多显示器支持优化规格

## 功能需求
优化 DailyLogger 在多显示器环境下的截图体验，支持选择捕获特定显示器、拼接多显示器截图，以及记录显示器配置信息。

## 优化范围
1. 支持选择捕获特定显示器（主显示器、副显示器或全部）
2. 支持拼接多显示器截图为一张全景图（可选功能）
3. 记录显示器配置信息（分辨率、位置、数量）
4. 前端设置界面支持显示器选择

## 不在范围内
- 不修改截图存储逻辑（仍保存为单张 PNG）
- 不修改 AI 分析流程
- 不支持显示器热插拔实时响应（下次捕获时自动检测）

## 验收条件（Given/When/Then）

### AC1 - 选择捕获特定显示器
- Given 用户有多台显示器
- When 用户在设置中选择捕获模式（主显示器/副显示器/全部）
- Then 系统仅捕获选定显示器的截图

### AC2 - 拼接多显示器截图
- Given 用户有多台显示器且选择"全部"模式
- When 自动捕获触发时
- Then 系统将所有显示器截图拼接为一张全景图

### AC3 - 记录显示器配置信息
- Given 截图捕获完成
- When 保存记录到数据库
- Then 记录中包含当前显示器配置（数量、分辨率、布局）

### AC4 - 前端设置界面支持
- Given 用户打开设置界面
- When 显示器设置部分展开
- Then 显示当前连接的显示器列表，支持选择捕获模式

## 技术约束
- 只使用 TailwindCSS（无内联样式，无 per-component CSS）
- 前端测试：`npm run test`（Vitest）须通过
- Rust 测试：`cargo test` 须通过
- 使用 `xcap` crate（macOS/Linux）和 `windows_capture` crate（Windows）

## 接口定义

### 前端组件
- `SettingsModal.vue` - 添加显示器选择设置

### 后端 API（新增）
- `get_monitors()` - 获取当前连接的显示器列表
- 修改 `capture_screen()` - 支持指定显示器捕获

### 数据库扩展
- `settings` 表添加 `capture_mode` 字段（primary/secondary/all）
- `settings` 表添加 `selected_monitor_index` 字段
- `records` 表可选添加 `monitor_info` 字段（JSON）

## 依赖
- SMART-001: 应用窗口识别（已完成窗口检测，可复用显示器枚举逻辑）
- SMART-002: 静默时段智能调整（可复用设置扩展模式）