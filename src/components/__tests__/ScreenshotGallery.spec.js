import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
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

    return wrapper
  }

  describe('AC1 - View Toggle', () => {
    it('renders view toggle buttons in header', async () => {
      const wrapper = await mountGallery()

      // Find view toggle buttons - looking for buttons with grid/list text
      const buttons = wrapper.findAll('button')
      const toggleButtons = buttons.filter(btn =>
        btn.text().includes('网格') || btn.text().includes('列表')
      )

      expect(toggleButtons.length).toBeGreaterThanOrEqual(2)
    })

    it('defaults to grid view on mount', async () => {
      const wrapper = await mountGallery()

      // Check for grid layout class (grid-cols)
      const gridContainer = wrapper.find('.grid')
      expect(gridContainer.exists()).toBe(true)
    })

    it('toggles to list view when list button is clicked', async () => {
      const wrapper = await mountGallery()

      // Find list view toggle button
      const buttons = wrapper.findAll('button')
      const listButton = buttons.find(btn => btn.text().includes('列表'))

      expect(listButton).toBeDefined()
      await listButton.trigger('click')
      await nextTick()

      // Should show list layout instead of grid
      const gridContainer = wrapper.find('.grid')
      const listContainer = wrapper.find('.divide-y')

      // Grid should be gone and list structure exists
      expect(gridContainer.exists()).toBe(false)
      expect(listContainer.exists()).toBe(true)
    })

    it('toggles back to grid view when grid button is clicked', async () => {
      const wrapper = await mountGallery()

      // First switch to list view
      const buttons = wrapper.findAll('button')
      const listButton = buttons.find(btn => btn.text().includes('列表'))

      await listButton.trigger('click')
      await nextTick()

      // Then switch back to grid
      const gridButton = buttons.find(btn => btn.text().includes('网格'))

      await gridButton.trigger('click')
      await nextTick()

      // Should show grid layout again
      const gridContainer = wrapper.find('.grid')
      expect(gridContainer.exists()).toBe(true)
    })

    it('grid view shows 3 columns layout', async () => {
      const wrapper = await mountGallery()

      // Check for grid container
      const gridContainer = wrapper.find('.grid')
      expect(gridContainer.exists()).toBe(true)

      // Should have responsive grid classes for 3 columns
      const classes = gridContainer.classes()
      const hasThreeColumns = classes.some(c =>
        c === 'grid-cols-3' ||
        c.includes('lg:grid-cols-3') ||
        c.includes('md:grid-cols-3')
      )
      expect(hasThreeColumns).toBe(true)
    })

    it('list view shows detailed information', async () => {
      const wrapper = await mountGallery()

      // Switch to list view
      const buttons = wrapper.findAll('button')
      const listButton = buttons.find(btn => btn.text().includes('列表'))

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
      const gridButton = buttons.find(btn => btn.text().includes('网格'))

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
      // Should contain formatted time (formatTimeShort outputs HH:MM:SS)
      expect(html.includes('09:00') || html.includes('09:05') || html.includes('09:10')).toBe(true)
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

      // Find and click the first screenshot card in grid view
      const gridContainer = wrapper.find('.grid')
      const cards = gridContainer.findAll('[class*="cursor-pointer"]')

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

      const gridContainer = wrapper.find('.grid')
      const cards = gridContainer.findAll('[class*="cursor-pointer"]')

      await cards[0].trigger('click')
      await nextTick()

      // Verify the selected screenshot has correct path
      expect(wrapper.vm.selectedScreenshot.screenshot_path).toBe('/path/screenshot1.png')
    })

    it('clicking in list view also opens modal', async () => {
      const wrapper = await mountGallery()

      // Switch to list view
      const buttons = wrapper.findAll('button')
      const listButton = buttons.find(btn => btn.text().includes('列表'))
      await listButton.trigger('click')
      await nextTick()

      // Find and click a list item
      const listContainer = wrapper.find('.divide-y')
      const listItems = listContainer.findAll('[class*="cursor-pointer"]')

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
      const gridContainer = wrapper.find('.grid')
      const cards = gridContainer.findAll('[class*="cursor-pointer"]')
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
      const gridContainer = wrapper.find('.grid')
      const cards = gridContainer.findAll('[class*="cursor-pointer"]')
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

      const gridContainer = wrapper.find('.grid')
      const cards = gridContainer.findAll('[class*="cursor-pointer"]')
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

      return wrapper
    }

    it('shows load more button when records exceed page size', async () => {
      const wrapper = await mountGalleryWithManyRecords(25)

      // 25 records with pageSize 20 should show load more button
      expect(wrapper.vm.hasMorePages).toBe(true)
      expect(wrapper.html()).toContain('加载更多')
    })

    it('hides load more button when all records are shown', async () => {
      const wrapper = await mountGalleryWithManyRecords(15)

      // 15 records with pageSize 20 should NOT show load more button
      expect(wrapper.vm.hasMorePages).toBe(false)
      expect(wrapper.html()).not.toContain('加载更多')
    })

    it('initially shows only first page of records', async () => {
      const wrapper = await mountGalleryWithManyRecords(25)

      // Should show only 20 items initially (first page)
      expect(wrapper.vm.paginatedScreenshots.length).toBe(20)
    })

    it('loads next page when clicking load more button', async () => {
      const wrapper = await mountGalleryWithManyRecords(25)

      // Initial state
      expect(wrapper.vm.currentPage).toBe(1)
      expect(wrapper.vm.paginatedScreenshots.length).toBe(20)

      // Click load more
      const loadMoreBtn = wrapper.findAll('button').find(btn => btn.text().includes('加载更多'))
      await loadMoreBtn.trigger('click')
      await nextTick()

      // Should now show all 25 items
      expect(wrapper.vm.currentPage).toBe(2)
      expect(wrapper.vm.paginatedScreenshots.length).toBe(25)
    })

    it('resets pagination when filter is applied', async () => {
      const wrapper = await mountGalleryWithManyRecords(25)

      // Load more to advance page
      const loadMoreBtn = wrapper.findAll('button').find(btn => btn.text().includes('加载更多'))
      await loadMoreBtn.trigger('click')
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

      const filterButton = wrapper.findAll('button').find(btn => btn.text().includes('筛选'))
      await filterButton.trigger('click')

      // Wait for the async applyFilter to complete
      await waitFor(() => wrapper.vm.currentPage === 1)

      expect(wrapper.vm.currentPage).toBe(1)
    })

    it('shows remaining count in load more button', async () => {
      const wrapper = await mountGalleryWithManyRecords(25)

      const loadMoreBtn = wrapper.findAll('button').find(btn => btn.text().includes('加载更多'))
      expect(loadMoreBtn.text()).toContain('5') // 25 - 20 = 5 remaining
    })
  })

  describe('AC5 - Meta Info Display', () => {
    it('truncates AI summary to 50 characters', async () => {
      const longContent = JSON.stringify({
        current_focus: 'This is a very long focus text that should be truncated to 50 characters maximum',
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

      const truncated = wrapper.vm.parseContent(longContent)
      expect(truncated.length).toBeLessThanOrEqual(50)
    })

    it('shows timestamp on each screenshot card', async () => {
      const wrapper = await mountGallery()

      // Each card should show a timestamp
      const html = wrapper.html()
      expect(html).toMatch(/\d{2}:\d{2}:\d{2}/) // HH:MM:SS format
    })
  })
})