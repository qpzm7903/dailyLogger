# FEAT-001: 支持重新分析已分析的截图记录

## 背景

用户反馈 (#53): 已经分析过的截图记录应该支持重新分析，因为用户可能更换了 AI 模型，或者想用不同的 prompt 重新生成分析结果。

## 需求

### 功能需求

1. 用户可以在截图记录详情中点击"重新分析"按钮
2. 系统调用当前配置的 AI 模型重新分析截图
3. 分析完成后，更新该记录的 content 字段
4. 显示重新分析进度和结果

### 非功能需求

- 重新分析操作不阻塞 UI
- 支持批量重新分析（可选）
- 记录重新分析的时间戳

## 技术方案

### 后端实现

**新增 Tauri 命令**: `reanalyze_record`

```rust
// src-tauri/src/auto_perception/mod.rs

#[tauri::command]
pub async fn reanalyze_record(
    record_id: i64,
    app: AppHandle,
) -> Result<ScreenAnalysis, String> {
    // 1. 从数据库获取记录
    // 2. 读取截图文件
    // 3. 调用 AI 分析
    // 4. 更新记录的 content 字段
    // 5. 返回新的分析结果
}
```

**数据库更新**: 在 `records` 表中更新 content 字段

### 前端实现

**修改文件**: `src/components/ScreenshotModal.vue`

- 在模态框底部添加"重新分析"按钮
- 点击后调用 `reanalyze_record` 命令
- 显示加载状态
- 分析完成后更新显示

### API 接口

```typescript
// src/types/tauri.ts
export interface ReanalyzeResult {
  record_id: number
  analysis: ScreenAnalysis
  reanalyzed_at: string
}

// Tauri command
invoke<ScreenAnalysis>('reanalyze_record', { recordId: number })
```

## 验收标准

### Given/When/Then

**场景 1: 单条记录重新分析**
- Given: 用户查看一条已分析的截图记录
- When: 用户点击"重新分析"按钮
- Then: 系统显示加载状态，调用 AI 分析，更新记录内容

**场景 2: 离线记录重新分析**
- Given: 用户有一条离线模式下创建的记录（content 包含 offline_pending）
- When: 用户点击"重新分析"按钮
- Then: 系统调用 AI 分析，更新 content，移除 offline_pending 标志

**场景 3: 分析失败处理**
- Given: 用户点击"重新分析"
- When: AI 调用失败（网络错误、API 错误等）
- Then: 显示错误提示，原有分析结果保留

## 影响范围

| 文件 | 修改类型 |
|------|----------|
| `src-tauri/src/auto_perception/mod.rs` | 新增 reanalyze_record 命令 |
| `src-tauri/src/main.rs` | 注册新命令 |
| `src/components/ScreenshotModal.vue` | 添加重新分析按钮 |
| `src/types/tauri.ts` | 新增类型定义 |
| `src/i18n/locales/zh.json` | 新增国际化文本 |
| `src/i18n/locales/en.json` | 新增国际化文本 |

## 测试用例

### 后端测试

```rust
#[test]
fn test_reanalyze_record_success() {
    // 1. 创建测试记录
    // 2. 调用 reanalyze_record
    // 3. 验证 content 更新
}

#[test]
fn test_reanalyze_record_not_found() {
    // 验证记录不存在时的错误处理
}
```

### 前端测试

```typescript
describe('ScreenshotModal reanalyze', () => {
  it('should show reanalyze button for analyzed record', () => {})
  it('should call reanalyze_record on click', () => {})
  it('should show loading state during analysis', () => {})
  it('should update content after successful analysis', () => {})
  it('should show error toast on failure', () => {})
})
```

## 估时

- 后端实现: 1.5pts
- 前端实现: 1pt
- 测试: 0.5pts
- **总计: 3pts**