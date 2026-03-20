# UX-012 HistoryViewer 虚拟滚动

**版本**: v1.41.0
**优先级**: MEDIUM

## 功能需求

为 HistoryViewer 组件的历史记录列表实现虚拟滚动，只渲染当前视口内的条目。当历史记录数量超过 200 条时，避免一次性渲染大量 DOM 节点导致的性能下降和内存占用过高。

**当前问题**:
- 全量渲染所有历史记录（可能数千条）
- 数据量大时滚动卡顿，初始渲染慢
- 内存占用随数据量线性增长

## 不在范围内

- 不修改历史记录的数据获取逻辑（Rust 后端分页可作后续优化）
- 不修改单条记录的展示内容和样式
- 不实现无限滚动加载（数据仍全量加载到内存，只是渲染虚拟化）

## 接口定义

### 虚拟滚动参数

```typescript
// 虚拟滚动配置
const VIRTUAL_SCROLL_CONFIG = {
  itemHeight: 80,          // 每条记录固定高度（px）
  overscan: 5,             // 视口外预渲染条目数（上下各 5 条）
  threshold: 100,          // 超过此数量时启用虚拟滚动
}
```

### 实现方案

优先使用 `@tanstack/vue-virtual`（Vue 官方推荐的虚拟化库）：

```typescript
import { useVirtualizer } from '@tanstack/vue-virtual'

const parentRef = ref<HTMLElement | null>(null)
const virtualizer = useVirtualizer({
  count: filteredRecords.value.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => VIRTUAL_SCROLL_CONFIG.itemHeight,
  overscan: VIRTUAL_SCROLL_CONFIG.overscan,
})
```

### 条目高度要求

虚拟滚动要求条目高度稳定可预测：
- 普通文本条目：固定 80px
- 带截图条目：固定 96px（含缩略图行）
- 如需支持动态高度，使用 `measureElement` 回调

## 验收条件（Given/When/Then）

### AC1 - 大数据量下 DOM 节点数量受控

- Given 历史记录有 1000 条数据
- When 用户打开 HistoryViewer
- Then DOM 中实际渲染的列表条目不超过 30 条（视口高度/条目高度 + overscan * 2）

### AC2 - 滚动流畅

- Given 历史记录有 500 条数据
- When 用户快速滚动列表
- Then 滚动帧率不低于 50fps，无明显卡顿（通过目测或 Performance API 验证）

### AC3 - 数据量少时不启用虚拟滚动

- Given 历史记录少于 100 条
- When 用户打开 HistoryViewer
- Then 正常渲染所有条目（不使用虚拟化），避免小数据量的额外开销

### AC4 - 过滤后虚拟滚动正确工作

- Given 用户通过标签或关键词过滤历史记录
- When 过滤结果仍有 200+ 条
- Then 虚拟滚动基于过滤后的数据正确渲染

### AC5 - 滚动位置不跳动

- Given 虚拟滚动列表正在显示
- When 新数据追加到列表顶部
- Then 用户当前滚动位置保持稳定，不发生跳动

## 技术约束

- 使用 `@tanstack/vue-virtual`（不自行实现虚拟滚动逻辑）
- 条目高度须在实现前测量并固定，确保滚动高度计算准确
- 前端测试须覆盖：虚拟化触发阈值逻辑、过滤后条目数量计算
- `npm run test` 全部通过
- 不允许因引入虚拟滚动导致现有测试失败
