/**
 * Virtual scroll composable for efficient rendering of large lists
 * Only renders visible items + buffer for smooth 60fps scrolling
 */

import { ref, computed, onMounted, onBeforeUnmount, type Ref } from 'vue'

export interface VirtualScrollOptions<T> {
  itemHeight: number // Estimated height of each item in pixels
  containerRef: Ref<HTMLElement | null>
  items: Ref<T[]>
  buffer?: number // Number of items to render outside visible area
}

export interface VirtualItem<T> {
  index: number
  data: T
  style: {
    position: 'absolute'
    transform: string
    width: string
  }
}

export function useVirtualScroll<T>(options: VirtualScrollOptions<T>) {
  const { itemHeight, containerRef, items, buffer = 5 } = options

  const scrollTop = ref(0)
  const containerHeight = ref(0)
  let rafId: number | null = null

  // Calculate visible range based on scroll position
  const visibleRange = computed(() => {
    const start = Math.floor(scrollTop.value / itemHeight)
    const visibleCount = Math.ceil(containerHeight.value / itemHeight)

    const startIndex = Math.max(0, start - buffer)
    const endIndex = Math.min(items.value.length, start + visibleCount + buffer)

    return { startIndex, endIndex }
  })

  // Get only the visible items with their positions
  const visibleItems = computed<VirtualItem<T>[]>(() => {
    const { startIndex, endIndex } = visibleRange.value

    return items.value.slice(startIndex, endIndex).map((data, i) => {
      const realIndex = startIndex + i
      return {
        index: realIndex,
        data,
        style: {
          position: 'absolute' as const,
          transform: `translateY(${realIndex * itemHeight}px)`,
          width: '100%'
        }
      }
    })
  })

  // Total height for scroll container (enables proper scrollbar)
  const totalHeight = computed(() => items.value.length * itemHeight)

  // Handle scroll with requestAnimationFrame for 60fps
  const handleScroll = () => {
    if (rafId !== null) return

    rafId = requestAnimationFrame(() => {
      if (containerRef.value) {
        scrollTop.value = containerRef.value.scrollTop
        containerHeight.value = containerRef.value.clientHeight
      }
      rafId = null
    })
  }

  // Update container height on resize
  const updateContainerHeight = () => {
    if (containerRef.value) {
      containerHeight.value = containerRef.value.clientHeight
    }
  }

  let resizeObserver: ResizeObserver | null = null

  onMounted(() => {
    if (containerRef.value) {
      containerHeight.value = containerRef.value.clientHeight
      containerRef.value.addEventListener('scroll', handleScroll, { passive: true })

      // Observe container resize (only in browser environment)
      if (typeof ResizeObserver !== 'undefined') {
        resizeObserver = new ResizeObserver(() => {
          updateContainerHeight()
        })
        resizeObserver.observe(containerRef.value)
      }
    }
  })

  onBeforeUnmount(() => {
    if (containerRef.value) {
      containerRef.value.removeEventListener('scroll', handleScroll)
    }
    if (resizeObserver) {
      resizeObserver.disconnect()
    }
    if (rafId !== null) {
      cancelAnimationFrame(rafId)
    }
  })

  // Scroll to a specific index
  const scrollToIndex = (index: number) => {
    if (containerRef.value) {
      containerRef.value.scrollTop = index * itemHeight
    }
  }

  return {
    visibleItems,
    totalHeight,
    visibleRange,
    scrollToIndex
  }
}
