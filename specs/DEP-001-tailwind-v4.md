# DEP-001: Tailwind CSS v4 升级

## 背景

Tailwind CSS v4 是一次重大架构升级，带来显著性能提升和新特性。当前项目使用 v3.4.19，需要升级到 v4.2.2。

## 目标

将 Tailwind CSS 从 v3.4.19 升级到 v4.2.2，获得性能提升和新特性。

## 迁移范围

### 1. 依赖更新
- `tailwindcss`: 3.4.19 → 4.2.2
- 新增: `@tailwindcss/vite` 插件

### 2. 配置迁移
- `tailwind.config.js` → CSS `@theme` 块
- 自定义颜色: `primary`, `secondary`, `dark`, `darker`

### 3. 样式入口更新
- `src/styles/main.css`:
  - 移除: `@tailwind base; @tailwind components; @tailwind utilities;`
  - 改为: `@import "tailwindcss";`

### 4. 类名更新
- `bg-gradient-to-r` → `bg-linear-to-r` (2处)
- 文件: `src/components/settings/BasicSettings.vue`

## 验收条件

### AC1: 所有测试通过
- Given: Tailwind CSS v4 已安装配置
- When: 运行测试
- Then: 所有 583 个前端测试 + 435 个 Rust 测试通过

### AC2: 样式正确渲染
- Given: 应用启动
- When: 查看设置页面
- Then: 渐变按钮样式正确显示

### AC3: CI 通过
- Given: 代码已推送
- When: CI 运行完成
- Then: 所有 workflow 通过

### AC4: 构建成功
- Given: Tailwind CSS v4 已配置
- When: 运行 `npm run build`
- Then: 构建成功无错误

## 风险评估

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| 渐变类名变更 | 低 | 项目中仅 2 处使用，手动更新 |
| 配置迁移 | 低 | 自定义颜色简单，手动迁移 |
| 构建工具兼容 | 中 | 使用官方 @tailwindcss/vite 插件 |

## 实施步骤

1. 安装 `tailwindcss@4` 和 `@tailwindcss/vite`
2. 更新 `vite.config.ts` 添加 Tailwind 插件
3. 更新 `src/styles/main.css` 使用 `@import "tailwindcss"`
4. 迁移自定义颜色到 CSS `@theme` 块
5. 更新 `bg-gradient-to-r` 为 `bg-linear-to-r`
6. 删除 `tailwind.config.js`
7. 运行测试验证
8. 提交并验证 CI

## 参考资料

- [Tailwind CSS v4 迁移指南](https://tailwindcss.com/blog/tailwindcss-v4)
- [Vite 插件文档](https://tailwindcss.com/docs/installation/vite)