import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick, ref, computed } from 'vue'
import ScreenshotGallery from '../ScreenshotGallery.vue'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

import { invoke } from '@tauri-apps/api/core'

// Mock ScreenshotModal component
vi.mock('./ScreenshotModal.vue', () => ({
  default: {
    name: 'ScreenshotModal',
    template: '<div class="mock-screenshot-modal"></div>',
    props: ['record']
  }
}))

// Mock useVirtualScroll to bypass virtual scrolling in tests
// Returns all items as visible so tests can interact with DOM elements
vi.mock('../composables/useVirtualScroll', () => ({
  useVirtualScroll: vi.fn((options) => {
    const ITEM_HEIGHT = 220
    const items = options.items

    const visibleItems = computed(() => {
      // Return all items as visible for testing purposes
      return items.value.map((data, i) => ({
        index: i,
        data,
        style: {
          position: 'absolute' as const,
          transform: `translateY(${i * ITEM_HEIGHT}px)`,
          width: '100%'
        }
      }))
    })

    const totalHeight = computed(() => items.value.length * ITEM_HEIGHT)

    return {
      visibleItems,
      totalHeight,
      visibleRange: computed(() => ({ startIndex: 0, endIndex: items.value.length })),
      scrollToIndex: vi.fn()
    }
  })
}))

describe('ScreenshotGallery', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  const mockScreenshots = [
    {
      id: 1,
      timestamp: '2026-03-14T09:00:00Z',
      source_type: 'auto',
      screenshot_path: '/path/screenshot1.png',
      content: JSON.stringify({ current_focus: 'VS Code', active_software: 'Code Editor' }),
      thumbnail: 'data:image/png;base64,test1'
    },
    {
      id: 2,
      timestamp: '2026-03-14T09:05:00Z',
      source_type: 'auto',
      screenshot_path: '/path/screenshot2.png',
      content: JSON.stringify({ current_focus: 'Chrome', active_software: 'Browser' }),
      thumbnail: 'data:image/png;base64,test2'
    },
    {
      id: 3,
      timestamp: '2026-03-14T09:10:00Z',
      source_type: 'auto',
      screenshot_path: '/path/screenshot3.png',
      content: JSON.stringify({ current_focus: 'Terminal', active_software: 'Command Line' }),
      thumbnail: 'data:image/png;base64,test3'
    }
  ]

  // Helper to wait for condition
  const waitFor = async (condition, timeout = 3000) => {
    const start = Date.now()
    while (!condition() && Date.now() - start < timeout) {
      await new Promise(resolve => setTimeout(resolve, 50))
      await nextTick()
    }
    if (!condition()) {
      throw new Error('waitFor timeout')
    }
  }

  // Helper to mount component with default settings
  const mountGallery = async () => {
    // Mock get_today_records to return screenshot records
    invoke.mockImplementation(async (cmd, args) => {
      if (cmd === 'get_today_records') {
        return mockScreenshots.map(s => ({
          id: s.id,
          timestamp: s.timestamp,
          source_type: s.source_type,
          screenshot_path: s.screenshot_path,
          content: s.content
        }))
      }
      if (cmd === 'get_screenshot') {
        return 'data:image/png;base64,testthumbnail'
      }
      return null
    })

    const wrapper = mount(ScreenshotGallery, {
      global: {
        stubs: {
          ScreenshotModal: true
        }
      }
    })

    // Wait for screenshots to load by checking vm.screenshots.length
    await waitFor(() => wrapper.vm.screenshots.length > 0)

    // Extra ticks for reactivity
    await nextTick()
    await nextTick()

    // Set scroll container height so virtual scroll renders visible items
    // (In test environment containerHeight is 0, causing visibleItems to be empty)
    if (wrapper.vm.scrollContainer) {
      Object.defineProperty(wrapper.vm.scrollContainer, 'clientHeight', {
        value: 1000,
        writable: true
      })
      // Trigger scroll event to update virtual scroll's containerHeight
      wrapper.vm.scrollContainer.dispatchEvent(new Event('scroll'))
    }

    // Wait for visible items to be computed
    await nextTick()
    await nextTick()

    return wrapper
  }

  describe('AC1 - View Toggle', () => {
    it('renders view toggle buttons in header', async () => {
      const wrapper = await mountGallery()

      // Find view toggle buttons - looking for buttons with grid/list text
      const buttons = wrapper.findAll('button')
      const toggleButtons = buttons.filter(btn =>
        btn.text().includes('Grid') || btn.text().includes('List')
      )

      expect(toggleButtons.length).toBeGreaterThanOrEqual(2)
    })

    it('defaults to grid view on mount', async () => {
      const wrapper = await mountGallery()

      // Check that screenshot items are rendered (indicating grid view is working)
      const items = wrapper.findAll('[class*="cursor-pointer"]')
      expect(items.length).toBeGreaterThan(0)
      // Verify view mode is grid
      expect(wrapper.vm.viewMode).toBe('grid')
    })

    it('toggles to list view when list button is clicked', async () => {
      const wrapper = await mountGallery()

      // Find list view toggle button
      const buttons = wrapper.findAll('button')
      const listButton = buttons.find(btn => btn.text().includes('List'))

      expect(listButton).toBeDefined()
      await listButton.trigger('click')
      await nextTick()

      // Should show list layout - verify view mode changed
      expect(wrapper.vm.viewMode).toBe('list')
      // List items should be present
      const items = wrapper.findAll('[class*="cursor-pointer"]')
      expect(items.length).toBeGreaterThan(0)
    })

    it('toggles back to grid view when grid button is clicked', async () => {
      const wrapper = await mountGallery()

      // First switch to list view
      const buttons = wrapper.findAll('button')
      const listButton = buttons.find(btn => btn.text().includes('List'))

      await listButton.trigger('click')
      await nextTick()

      // Then switch back to grid
      const gridButton = buttons.find(btn => btn.text().includes('Grid'))

      await gridButton.trigger('click')
      await nextTick()

      // Should show grid layout again
      expect(wrapper.vm.viewMode).toBe('grid')
      const items = wrapper.findAll('[class*="cursor-pointer"]')
      expect(items.length).toBeGreaterThan(0)
    })

    it('grid view shows screenshot cards', async () => {
      const wrapper = await mountGallery()

      // Grid view should have screenshot cards
      const items = wrapper.findAll('[class*="cursor-pointer"]')
      expect(items.length).toBeGreaterThan(0)
    })

    it('list view shows detailed information', async () => {
      const wrapper = await mountGallery()

      // Switch to list view
      const buttons = wrapper.findAll('button')
      const listButton = buttons.find(btn => btn.text().includes('List'))

      await listButton.trigger('click')
      await nextTick()

      // List view should have row structure with details
      const html = wrapper.html()
      // Should have time column
      expect(html.includes('时间') || html.includes('Time') ||
             html.includes('09:00') || html.includes('09:05')).toBe(true)
      // Should have action/view column
      expect(html.includes('查看') || html.includes('View') ||
             html.includes('操作')).toBe(true)
    })

    it('active view button is highlighted', async () => {
      const wrapper = await mountGallery()

      // Find the grid button and check it's highlighted (active state)
      const buttons = wrapper.findAll('button')
      const gridButton = buttons.find(btn => btn.text().includes('Grid'))

      expect(gridButton).toBeDefined()
      // Grid should be active by default (bg-primary class)
      expect(gridButton.classes().includes('bg-primary')).toBe(true)
    })
  })

  describe('Screenshot rendering', () => {
    it('displays screenshots with thumbnails', async () => {
      const wrapper = await mountGallery()

      const images = wrapper.findAll('img')
      expect(images.length).toBe(mockScreenshots.length)
    })

    it('shows timestamp on each screenshot card', async () => {
      const wrapper = await mountGallery()

      const html = wrapper.html()
      // Compute expected local time from first mock timestamp (avoids timezone hardcoding)
      const d = new Date(mockScreenshots[0].timestamp)
      const expectedHH = String(d.getHours()).padStart(2, '0')
      const expectedMM = String(d.getMinutes()).padStart(2, '0')
      expect(html.includes(`${expectedHH}:${expectedMM}`)).toBe(true)
    })

    it('opens modal when clicking on screenshot', async () => {
      const wrapper = await mountGallery()

      // Click on first screenshot card
      const cards = wrapper.findAll('[class*="cursor-pointer"]')
      if (cards.length > 0) {
        await cards[0].trigger('click')
        await nextTick()

        // Modal should be visible
        expect(wrapper.vm.showDetail).toBe(true)
      }
    })
  })

  describe('AC3 - Quick Preview Modal', () => {
    it('clicking thumbnail opens ScreenshotModal with correct record', async () => {
      const wrapper = await mountGallery()

      // Find screenshot cards in grid view
      const cards = wrapper.findAll('[class*="cursor-pointer"]')

      expect(cards.length).toBeGreaterThan(0)

      // Click first card
      await cards[0].trigger('click')
      await nextTick()

      // Verify modal state
      expect(wrapper.vm.showDetail).toBe(true)
      expect(wrapper.vm.selectedScreenshot).toBeTruthy()
      expect(wrapper.vm.selectedScreenshot.id).toBe(1)
    })

    it('passes correct screenshot_path to modal', async () => {
      const wrapper = await mountGallery()

      const cards = wrapper.findAll('[class*="cursor-pointer"]')

      await cards[0].trigger('click')
      await nextTick()

      // Verify the selected screenshot has correct path
      expect(wrapper.vm.selectedScreenshot.screenshot_path).toBe('/path/screenshot1.png')
    })

    it('clicking in list view also opens modal', async () => {
      const wrapper = await mountGallery()

      // Switch to list view
      const buttons = wrapper.findAll('button')
      const listButton = buttons.find(btn => btn.text().includes('List'))
      await listButton.trigger('click')
      await nextTick()

      // Find and click a list item
      const listItems = wrapper.findAll('[class*="cursor-pointer"]')

      expect(listItems.length).toBeGreaterThan(0)

      await listItems[1].trigger('click')
      await nextTick()

      // Verify modal opened with correct record
      expect(wrapper.vm.showDetail).toBe(true)
      expect(wrapper.vm.selectedScreenshot.id).toBe(2)
    })

    it('ScreenshotModal component is rendered when showDetail is true', async () => {
      const wrapper = await mountGallery()

      // Initially modal should not be visible
      expect(wrapper.findComponent({ name: 'ScreenshotModal' }).exists()).toBe(false)

      // Click to open modal
      const cards = wrapper.findAll('[class*="cursor-pointer"]')
      await cards[0].trigger('click')
      await nextTick()

      // Now modal should be rendered
      const modal = wrapper.findComponent({ name: 'ScreenshotModal' })
      expect(modal.exists()).toBe(true)
      expect(modal.props('record')).toEqual(wrapper.vm.selectedScreenshot)
    })

    it('closing modal resets showDetail state', async () => {
      const wrapper = await mountGallery()

      // Open modal
      const cards = wrapper.findAll('[class*="cursor-pointer"]')
      await cards[0].trigger('click')
      await nextTick()

      expect(wrapper.vm.showDetail).toBe(true)

      // Find modal and emit close event
      const modal = wrapper.findComponent({ name: 'ScreenshotModal' })
      await modal.vm.$emit('close')
      await nextTick()

      // Modal should be hidden
      expect(wrapper.vm.showDetail).toBe(false)
    })

    it('modal record includes content for AI analysis display', async () => {
      const wrapper = await mountGallery()

      const cards = wrapper.findAll('[class*="cursor-pointer"]')
      await cards[0].trigger('click')
      await nextTick()

      // Verify the record passed to modal has content
      const selected = wrapper.vm.selectedScreenshot
      expect(selected.content).toBeTruthy()
      expect(selected.content).toContain('current_focus')
    })
  })

  describe('AC4 - Pagination', () => {
    // Generate 25 mock screenshots for pagination testing
    const generateMockScreenshots = (count) => {
      return Array.from({ length: count }, (_, i) => ({
        id: i + 1,
        timestamp: `2026-03-14T09:${String(i % 60).padStart(2, '0')}:00Z`,
        source_type: 'auto',
        screenshot_path: `/path/screenshot${i + 1}.png`,
        content: JSON.stringify({ current_focus: `Task ${i + 1}`, active_software: 'App' }),
        thumbnail: `data:image/png;base64,test${i + 1}`
      }))
    }

    const mountGalleryWithManyRecords = async (recordCount = 25) => {
      const manyMockScreenshots = generateMockScreenshots(recordCount)

      invoke.mockImplementation(async (cmd) => {
        if (cmd === 'get_today_records') {
          return manyMockScreenshots.map(s => ({
            id: s.id,
            timestamp: s.timestamp,
            source_type: s.source_type,
            screenshot_path: s.screenshot_path,
            content: s.content
          }))
        }
        if (cmd === 'get_screenshot') {
          return 'data:image/png;base64,testthumbnail'
        }
        return null
      })

      const wrapper = mount(ScreenshotGallery, {
        global: {
          stubs: {
            ScreenshotModal: true
          }
        }
      })

      await waitFor(() => wrapper.vm.screenshots.length > 0)
      await nextTick()
      await nextTick()

      // Set scroll container height so virtual scroll renders visible items
      if (wrapper.vm.scrollContainer) {
        Object.defineProperty(wrapper.vm.scrollContainer, 'clientHeight', {
          value: 1000,
          writable: true
        })
        // Trigger scroll event to update virtual scroll's containerHeight
        wrapper.vm.scrollContainer.dispatchEvent(new Event('scroll'))
      }

      // Wait for visible items to be computed
      await nextTick()
      await nextTick()

      return wrapper
    }

    it('initially shows only first page of records (20 items)', async () => {
      const wrapper = await mountGalleryWithManyRecords(25)

      // Should show only 20 items initially (first page)
      expect(wrapper.vm.paginatedScreenshots.length).toBe(20)
    })

    it('shows remaining count indicator when more records exist', async () => {
      const wrapper = await mountGalleryWithManyRecords(25)

      // Should show remaining count text
      expect(wrapper.vm.hasMorePages).toBe(true)
      const html = wrapper.html()
      expect(html.includes('remaining') || html.includes('Load More')).toBe(true)
    })

    it('hides remaining indicator when all records are shown', async () => {
      const wrapper = await mountGalleryWithManyRecords(15)

      // 15 records with pageSize 20 should NOT show load more
      expect(wrapper.vm.hasMorePages).toBe(false)
    })

    it('loads next page when loadMore is called', async () => {
      const wrapper = await mountGalleryWithManyRecords(25)

      // Initial state
      expect(wrapper.vm.currentPage).toBe(1)
      expect(wrapper.vm.paginatedScreenshots.length).toBe(20)

      // Call loadMore programmatically (simulating scroll trigger)
      wrapper.vm.loadMore()
      await nextTick()

      // Wait for the setTimeout to complete
      await new Promise(resolve => setTimeout(resolve, 200))
      await nextTick()

      // Should now show all 25 items
      expect(wrapper.vm.currentPage).toBe(2)
      expect(wrapper.vm.paginatedScreenshots.length).toBe(25)
    })

    it('resets pagination when filter is applied', async () => {
      const wrapper = await mountGalleryWithManyRecords(25)

      // Load more to advance page
      wrapper.vm.loadMore()
      await new Promise(resolve => setTimeout(resolve, 200))
      await nextTick()
      expect(wrapper.vm.currentPage).toBe(2)

      // Clear previous invoke calls and set up new mock for date range filter
      invoke.mockClear()
      invoke.mockImplementation(async (cmd) => {
        if (cmd === 'get_records_by_date_range') {
          return generateMockScreenshots(5)
        }
        if (cmd === 'get_screenshot') {
          return 'data:image/png;base64,test'
        }
        return null
      })

      const dateInputs = wrapper.findAll('input[type="date"]')
      await dateInputs[0].setValue('2026-03-10')
      await dateInputs[1].setValue('2026-03-11')

      const filterButton = wrapper.findAll('button').find(btn => btn.text().includes('Filter'))
      await filterButton.trigger('click')

      // Wait for the async applyFilter to complete
      await waitFor(() => wrapper.vm.currentPage === 1)

      expect(wrapper.vm.currentPage).toBe(1)
    })

    it('shows loading indicator when isLoadingMore is true', async () => {
      const wrapper = await mountGalleryWithManyRecords(25)

      // Set loading state
      wrapper.vm.isLoadingMore = true
      await nextTick()

      // Should show loading indicator
      expect(wrapper.html()).toContain('Loading...')
    })

    it('calculates correct remaining count', async () => {
      const wrapper = await mountGalleryWithManyRecords(45)

      // 45 records, first page shows 20, remaining should be 25
      expect(wrapper.vm.remainingCount).toBe(25)
    })

    it('does not load more when already at last page', async () => {
      const wrapper = await mountGalleryWithManyRecords(25)

      // Load all pages
      wrapper.vm.loadMore() // Page 2 - shows all 25
      await new Promise(resolve => setTimeout(resolve, 200))
      await nextTick()

      expect(wrapper.vm.currentPage).toBe(2)

      // Try to load more again - should not change
      wrapper.vm.loadMore()
      await new Promise(resolve => setTimeout(resolve, 200))
      await nextTick()

      expect(wrapper.vm.currentPage).toBe(2) // Should stay at 2, not go to 3
    })
  })

  describe('AC5 - Meta Info Display', () => {
    it('truncates AI summary to 50 characters with ellipsis when over limit', async () => {
      // Create content longer than 50 chars
      const longText = 'This is a very long focus text that definitely exceeds fifty characters and should be truncated'
      const longContent = JSON.stringify({
        current_focus: longText,
        active_software: 'Test App'
      })

      invoke.mockImplementation(async (cmd) => {
        if (cmd === 'get_today_records') {
          return [{
            id: 1,
            timestamp: '2026-03-14T09:00:00Z',
            source_type: 'auto',
            screenshot_path: '/path/screenshot.png',
            content: longContent
          }]
        }
        if (cmd === 'get_screenshot') {
          return 'data:image/png;base64,test'
        }
        return null
      })

      const wrapper = mount(ScreenshotGallery, {
        global: { stubs: { ScreenshotModal: true } }
      })

      await waitFor(() => wrapper.vm.screenshots.length > 0)
      await nextTick()
      await nextTick()

      // Set scroll container height so virtual scroll renders visible items
      if (wrapper.vm.scrollContainer) {
        Object.defineProperty(wrapper.vm.scrollContainer, 'clientHeight', {
          value: 1000,
          writable: true
        })
        // Trigger scroll event to update virtual scroll's containerHeight
        wrapper.vm.scrollContainer.dispatchEvent(new Event('scroll'))
      }

      // Wait for visible items to be computed
      await nextTick()
      await nextTick()

      const truncated = wrapper.vm.parseContent(longContent)
      // Should be exactly 50 chars + "..." = 53 total
      expect(truncated.length).toBe(53)
      expect(truncated.endsWith('...')).toBe(true)
    })

    it('does not add ellipsis when AI summary is within 50 characters', async () => {
      const shortText = 'Short text'
      const shortContent = JSON.stringify({
        current_focus: shortText,
        active_software: 'Test App'
      })

      invoke.mockImplementation(async (cmd) => {
        if (cmd === 'get_today_records') {
          return [{
            id: 1,
            timestamp: '2026-03-14T09:00:00Z',
            source_type: 'auto',
            screenshot_path: '/path/screenshot.png',
            content: shortContent
          }]
        }
        if (cmd === 'get_screenshot') {
          return 'data:image/png;base64,test'
        }
        return null
      })

      const wrapper = mount(ScreenshotGallery, {
        global: { stubs: { ScreenshotModal: true } }
      })

      await waitFor(() => wrapper.vm.screenshots.length > 0)
      await nextTick()
      await nextTick()

      // Set scroll container height so virtual scroll renders visible items
      if (wrapper.vm.scrollContainer) {
        Object.defineProperty(wrapper.vm.scrollContainer, 'clientHeight', {
          value: 1000,
          writable: true
        })
        // Trigger scroll event to update virtual scroll's containerHeight
        wrapper.vm.scrollContainer.dispatchEvent(new Event('scroll'))
      }

      // Wait for visible items to be computed
      await nextTick()
      await nextTick()

      const result = wrapper.vm.parseContent(shortContent)
      expect(result).toBe(shortText)
      expect(result.endsWith('...')).toBe(false)
    })

    it('shows timestamp in HH:mm:ss format on each screenshot card in grid view', async () => {
      const wrapper = await mountGallery()

      // Grid view should have timestamps in HH:mm:ss format
      const html = wrapper.html()

      // Compute expected local times from mock data to avoid timezone hardcoding
      const expectedPatterns = mockScreenshots.map(s => {
        const d = new Date(s.timestamp)
        return `${String(d.getHours()).padStart(2,'0')}:${String(d.getMinutes()).padStart(2,'0')}:${String(d.getSeconds()).padStart(2,'0')}`
      })
      const regex = new RegExp(expectedPatterns.join('|'))
      expect(html).toMatch(regex)
    })

    it('shows timestamp in HH:mm:ss format in list view', async () => {
      const wrapper = await mountGallery()

      // Switch to list view
      const buttons = wrapper.findAll('button')
      const listButton = buttons.find(btn => btn.text().includes('List'))
      await listButton.trigger('click')
      await nextTick()

      const html = wrapper.html()

      // Compute expected local times from mock data to avoid timezone hardcoding
      const expectedPatterns = mockScreenshots.map(s => {
        const d = new Date(s.timestamp)
        return `${String(d.getHours()).padStart(2,'0')}:${String(d.getMinutes()).padStart(2,'0')}:${String(d.getSeconds()).padStart(2,'0')}`
      })
      const regex = new RegExp(expectedPatterns.join('|'))
      expect(html).toMatch(regex)
    })
  })
})