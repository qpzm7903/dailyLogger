# TEST-018 前端 E2E 测试框架（Playwright + Vite Dev Server）

**版本**: v1.62.0
**优先级**: HIGH

## 功能需求

建立基于 Playwright 的前端 E2E 测试框架，对 Vite Dev Server 进行真实浏览器 UI 测试。通过 mock Tauri IPC 层（`window.__TAURI_INTERNALS__`），在不编译 Rust 后端的前提下，验证所有前端页面交互、modal 流程和用户工作流。

**目标**：`npm run test:e2e` 一条命令完成全部前端 E2E 验证，agent 可自动执行。

**现状问题**：

| 问题 | 说明 |
|------|------|
| 现有 `e2e-tests/` 仅 Windows 可用 | `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` 是 Windows WebView2 专属，macOS WKWebView 无 CDP 协议 |
| AI Agent 驱动非确定性 | LLM 决策不可重复，同一测试多次运行结果不一致，无法作为回归测试 |
| 未集成 CI | 需手动启动 Tauri 应用 + 手动运行 Python 脚本 |
| 依赖 OpenAI API | 每次测试消耗 API 费用，延迟 30-60 秒/步 |

**方案核心思路**：

Vue 前端通过 `invoke()` 调用 Rust 后端，这是一个清晰的边界。在浏览器中 mock 掉 `window.__TAURI_INTERNALS__`，Playwright 就能对 Vite Dev Server（localhost:1420）做真实浏览器 E2E 测试。后端逻辑由已有的 `cargo test`（435 个测试）覆盖。两层叠加约等于全链路覆盖。

## 不在范围内

- 不修改现有 `e2e-tests/` Python 框架（保留作为探索性测试工具）
- 不测试 Rust 后端逻辑（已由 cargo test 覆盖）
- 不需要编译完整 Tauri 应用
- 不引入 Cypress 或其他 E2E 框架
- 不测试平台特定功能（截图、系统托盘等）

## 架构设计

### 目录结构

```
tests/e2e/
├── playwright.config.ts          # Playwright 配置（webServer 自动启动 Vite）
├── fixtures/
│   ├── tauri-mock.ts             # Mock window.__TAURI_INTERNALS__
│   ├── test-data.ts              # 测试数据工厂
│   └── base-test.ts              # 扩展 Playwright test，自动注入 mock
├── pages/                        # Page Object Model
│   ├── main-page.ts              # 主页面（记录列表、header 按钮）
│   ├── quick-note-modal.ts       # 快速笔记弹窗
│   └── settings-modal.ts         # 设置弹窗（含 4 个子面板）
└── specs/
    ├── smoke.spec.ts             # 冒烟测试：应用能加载
    ├── quick-note.spec.ts        # 快速笔记流程
    ├── settings.spec.ts          # 设置流程
    ├── history.spec.ts           # 历史浏览
    ├── tag-filter.spec.ts        # 标签筛选
    ├── screenshot-gallery.spec.ts # 截图画廊
    ├── report.spec.ts            # 报告生成
    ├── export.spec.ts            # 导出功能
    ├── backup.spec.ts            # 备份恢复
    └── timeline.spec.ts          # 时间线
```

### Tauri IPC Mock 策略

分层 mock 架构：

```typescript
// 1. 默认 mock — 所有测试共享的安全默认返回值
const DEFAULT_MOCKS: Record<string, (args?: any) => any> = {
  'get_settings': () => FACTORY.settings(),
  'get_today_records': () => [],
  'get_network_status': () => ({ online: true }),
  'get_platform_info': () => ({ os: 'macos', arch: 'aarch64' }),
  'get_auto_capture_status': () => ({ running: false }),
  'get_all_manual_tags': () => [],
  // ... 覆盖全部 70+ 命令的安全默认值
};

// 2. 测试级覆盖 — 每个测试按需覆盖特定命令的返回值
// 3. 数据工厂 — 生成结构一致的测试数据
```

通过 `page.addInitScript()` 在页面加载前注入 mock，拦截所有 `invoke()` 调用。

### Dev Server 集成

Playwright 内置 `webServer` 配置，自动启动 `npm run dev` 并等待 `localhost:1420` 就绪，测试结束后自动关闭。不需要手动启动任何服务。

### Modal 导航测试模式

本应用无 URL 路由，全部 modal 导航。测试通过「点击按钮 → 等待 modal 出现 → 交互 → 验证」驱动，使用语义选择器（`getByRole`、`getByText`）而非 CSS 选择器。

## 测试场景优先级

| 优先级 | ID | 测试场景 | 涉及组件 | Mock 命令 |
|--------|-----|---------|---------|-----------|
| P0 | E2E-001 | 冒烟测试：应用加载、header 可见 | App.vue | get_settings, get_today_records |
| P0 | E2E-002 | 快速笔记：打开→输入→保存→列表显示 | QuickNoteModal | create_record, get_today_records |
| P0 | E2E-003 | 设置：打开→修改→保存→重开验证 | SettingsModal + 4 子面板 | get_settings, save_settings |
| P1 | E2E-004 | 历史浏览：打开→翻页→查看详情 | HistoryViewer | get_history_records |
| P1 | E2E-005 | 标签筛选：选标签→验证过滤→清除 | TagCloud, TagFilter | get_all_manual_tags, get_records_by_manual_tags |
| P1 | E2E-006 | 截图画廊：查看→点击放大 | ScreenshotGallery, ScreenshotModal | get_today_records |
| P2 | E2E-007 | 日报生成：触发→等待→查看结果 | DailySummaryViewer | generate_daily_summary |
| P2 | E2E-008 | 导出：选格式→导出→确认 | ExportModal | export_records |
| P2 | E2E-009 | 备份：创建→列表→恢复 | BackupModal | create_backup, list_backups |
| P3 | E2E-010 | 时间线：日期导航→记录展示 | TimelineVisualization | get_timeline_for_date |

## 接口定义

### npm scripts

```json
{
  "test:e2e": "playwright test --config tests/e2e/playwright.config.ts",
  "test:e2e:ui": "playwright test --config tests/e2e/playwright.config.ts --ui",
  "test:e2e:debug": "playwright test --config tests/e2e/playwright.config.ts --debug"
}
```

### Playwright 配置要点

```typescript
// tests/e2e/playwright.config.ts
export default defineConfig({
  testDir: './specs',
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 30000,
  },
  use: {
    baseURL: 'http://localhost:1420',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],
});
```

### 测试基础 Fixture

```typescript
// tests/e2e/fixtures/base-test.ts
import { test as base } from '@playwright/test';

type TauriMockOverrides = Record<string, (args?: any) => any>;

export const test = base.extend<{ tauriMock: TauriMockOverrides }>({
  tauriMock: [{}, { option: true }],
  page: async ({ page, tauriMock }, use) => {
    await page.addInitScript((mocks) => {
      // 注入默认 mock + 测试级覆盖
      window.__TAURI_INTERNALS__ = { /* ... */ };
    }, tauriMock);
    await use(page);
  },
});
```

## CI 集成

### GitHub Actions 新增 job

在 `.github/workflows/test.yml` 中新增 `test-e2e` job：

```yaml
test-e2e:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-node@v4
      with: { node-version: 20 }
    - run: npm ci
    - run: npx playwright install chromium --with-deps
    - run: npm run test:e2e
    - uses: actions/upload-artifact@v4
      if: always()
      with:
        name: playwright-report
        path: playwright-report/
```

**说明**：E2E 前端测试仅需 Chromium + Vite Dev Server，不需要 Rust 工具链，可在 Ubuntu runner 上运行（成本最低）。

## 验收条件（Given/When/Then）

### AC1 - 一条命令运行全部 E2E 测试

- Given 项目已 `npm install`
- When 执行 `npm run test:e2e`
- Then Vite Dev Server 自动启动，全部测试执行完毕，输出 pass/fail 结果

### AC2 - 快速笔记 E2E 流程通过

- Given 应用加载完成且 Tauri IPC 已 mock
- When 点击快速笔记按钮 → 输入文本 → 点击保存
- Then modal 关闭，记录列表显示新记录

### AC3 - 设置 E2E 流程通过

- Given 应用加载完成
- When 打开设置 → 修改 API URL → 保存 → 重新打开设置
- Then 修改后的值已保存（mock 的 save_settings 被调用且参数正确）

### AC4 - CI 中 E2E 测试自动运行

- Given PR 提交到 main 分支
- When GitHub Actions 触发
- Then test-e2e job 在 Ubuntu runner 上通过

### AC5 - 测试结果确定性

- Given 同一测试代码
- When 连续运行 3 次
- Then 3 次结果完全一致（无 flaky test）

### AC6 - i18n 兼容

- Given 应用支持中英文
- When 测试使用语义选择器（getByRole）和正则匹配（/保存|save/i）
- Then 测试在两种语言下均可通过

### AC7 - 不影响现有测试

- Given E2E 框架已集成
- When 执行 `npm run test`（Vitest 单元测试）
- Then 原有 905 个测试全部通过，不受影响

## 技术约束

- 使用 `@playwright/test` 作为测试框架，不引入其他 E2E 工具
- 所有测试文件使用 TypeScript
- 选择器优先使用 `getByRole()` > `getByText()` > `getByTestId()` > CSS 选择器
- Mock 返回值的类型必须与 `src/types/tauri.ts` 中定义的类型一致
- 测试超时不超过 30 秒/用例
- 不依赖任何外部 API（OpenAI 等）
- 现有 `e2e-tests/` Python 框架保持不变

## 风险评估

| 风险 | 等级 | 缓解方案 |
|------|------|---------|
| Mock 与真实 Tauri IPC 行为不一致 | HIGH | Mock 返回值结构严格遵循 Rust `#[tauri::command]` 的实际返回类型，用 TypeScript 类型约束 |
| i18n 切换导致选择器失效 | MEDIUM | 使用正则匹配双语文本，优先使用 getByRole 语义选择器 |
| Vite HMR 干扰测试稳定性 | LOW | CI 中 Vite 不开启 HMR；或使用 `vite preview`（生产构建） |
| 组件选择器随 UI 重构变动 | MEDIUM | Page Object Model 封装选择器，变动时只需修改 POM 一处 |

## 与现有测试体系的关系

```
测试金字塔（从底到顶）：
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  cargo test (435)     — Rust 后端逻辑
  npm run test (905)   — Vue 组件单元/集成测试（jsdom）
  npm run test:e2e     — 前端 E2E（真实浏览器 + mock IPC）  ← 新增
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  e2e-tests/ (Python)  — 探索性测试（AI Agent，手动运行）
```

Quality Gate 更新为：
```bash
cd src-tauri && cargo fmt && cargo clippy -- -D warnings && cargo test
npm run test
npm run test:e2e    # 新增
```
