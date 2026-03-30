import { describe, it, expect, beforeEach, vi } from 'vitest'
import { ref } from 'vue'
import { useVirtualScroll } from '../useVirtualScroll'

// Helper to create a mock container element
function createMockContainer(height = 500, initialScrollTop = 0): HTMLElement {
  let scrollTop = initialScrollTop
  const el = document.createElement('div')
  Object.defineProperty(el, 'clientHeight', { value: height, configurable: true })
  Object.defineProperty(el, 'scrollTop', {
    get: () => scrollTop,
    set: (v: number) => { scrollTop = v },
    configurable: true,
  })
  el.addEventListener = vi.fn()
  el.removeEventListener = vi.fn()
  document.body.appendChild(el)
  return el
}

describe('useVirtualScroll', () => {
  let containerRef: ReturnType<typeof ref<HTMLElement | null>>
  let items: ReturnType<typeof ref<{ id: number; text: string }[]>>

  beforeEach(() => {
    document.body.innerHTML = ''
    containerRef = ref<HTMLElement | null>(null)
    items = ref(
      Array.from({ length: 100 }, (_, i) => ({ id: i, text: `Item ${i}` }))
    )
  })

  it('returns correct totalHeight', () => {
    const container = createMockContainer()
    containerRef.value = container

    const { totalHeight } = useVirtualScroll({
      itemHeight: 50,
      containerRef,
      items,
    })

    expect(totalHeight.value).toBe(5000) // 100 items * 50px
  })

  it('returns visible items with default state (containerHeight=0 shows only buffer items)', () => {
    const container = createMockContainer(500)
    containerRef.value = container

    const { visibleItems } = useVirtualScroll({
      itemHeight: 50,
      containerRef,
      items,
      buffer: 2,
    })

    // Without onMounted firing (no Vue component context), containerHeight stays 0.
    // visibleCount = ceil(0/50) = 0, so range is max(0, 0-2)..min(100, 0+0+2) = 0..2
    expect(visibleItems.value.length).toBe(2)
    expect(visibleItems.value[0].data.id).toBe(0)
    expect(visibleItems.value[1].data.id).toBe(1)
  })

  it('computes correct translateY transform for each item', () => {
    const container = createMockContainer(500)
    containerRef.value = container

    const { visibleItems } = useVirtualScroll({
      itemHeight: 50,
      containerRef,
      items,
      buffer: 2,
    })

    const firstItem = visibleItems.value[0]
    expect(firstItem.index).toBe(0)
    expect(firstItem.data).toEqual({ id: 0, text: 'Item 0' })
    expect(firstItem.style.position).toBe('absolute')
    expect(firstItem.style.transform).toBe('translateY(0px)')
  })

  it('handles empty items array', () => {
    const container = createMockContainer()
    containerRef.value = container
    const emptyItems = ref<{ id: number; text: string }[]>([])

    const { visibleItems, totalHeight } = useVirtualScroll({
      itemHeight: 50,
      containerRef,
      items: emptyItems,
    })

    expect(totalHeight.value).toBe(0)
    expect(visibleItems.value).toEqual([])
  })

  it('preserves generic types through the composable', () => {
    const container = createMockContainer()
    containerRef.value = container

    interface CustomItem {
      id: number
      label: string
    }
    const customItems = ref<CustomItem[]>(
      Array.from({ length: 10 }, (_, i) => ({ id: i, label: `Label ${i}` }))
    )

    const { visibleItems } = useVirtualScroll({
      itemHeight: 50,
      containerRef,
      items: customItems,
    })

    // Type check: visibleItems should be VirtualItem<CustomItem>[]
    const first = visibleItems.value[0]
    if (first) {
      expect(first.data.label).toBe('Label 0')
    }
  })

  it('scrollToIndex sets scrollTop on container', () => {
    const container = createMockContainer()
    containerRef.value = container

    const { scrollToIndex } = useVirtualScroll({
      itemHeight: 50,
      containerRef,
      items,
    })

    scrollToIndex(10)
    expect(container.scrollTop).toBe(500) // 10 * 50px
  })

  it('handles scrollToIndex when container is null', () => {
    containerRef.value = null

    const { scrollToIndex } = useVirtualScroll({
      itemHeight: 50,
      containerRef,
      items,
    })

    // Should not throw
    expect(() => scrollToIndex(10)).not.toThrow()
  })

  it('handles small item count', () => {
    const container = createMockContainer(500)
    containerRef.value = container
    const smallItems = ref([{ id: 0, text: 'Only' }])

    const { visibleItems, totalHeight } = useVirtualScroll({
      itemHeight: 50,
      containerRef,
      items: smallItems,
    })

    expect(totalHeight.value).toBe(50)
    expect(visibleItems.value.length).toBe(1)
    expect(visibleItems.value[0].data.text).toBe('Only')
  })

  it('totalHeight updates when items change', () => {
    const container = createMockContainer()
    containerRef.value = container

    const { totalHeight } = useVirtualScroll({
      itemHeight: 50,
      containerRef,
      items,
    })

    expect(totalHeight.value).toBe(5000)
    items.value = items.value.slice(0, 20)
    expect(totalHeight.value).toBe(1000)
  })
})
