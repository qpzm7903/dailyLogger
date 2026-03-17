import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import DailySummaryViewer from '../DailySummaryViewer.vue'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@tauri-apps/plugin-shell', () => ({
  open: vi.fn()
}))

describe('DailySummaryViewer', () => {
  let invokeMock
  let openMock

  beforeEach(async () => {
    const { invoke } = await import('@tauri-apps/api/core')
    const { open } = await import('@tauri-apps/plugin-shell')
    invokeMock = invoke
    openMock = open
    invokeMock.mockClear()
    openMock.mockClear()
  })

  it('renders modal with title', () => {
    invokeMock.mockResolvedValue('Test content')
    const wrapper = mount(DailySummaryViewer, {
      props: {
        summaryPath: '/test/path/summary.md'
      }
    })
    expect(wrapper.text()).toContain('📝 日报预览')
  })

  it('shows loading state initially', () => {
    invokeMock.mockResolvedValue('Test content')
    const wrapper = mount(DailySummaryViewer, {
      props: {
        summaryPath: '/test/path/summary.md'
      }
    })
    expect(wrapper.text()).toContain('加载中...')
  })

  it('loads and displays summary content', async () => {
    const testContent = '# Daily Summary\n\nTest content here'
    invokeMock.mockResolvedValue(testContent)

    const wrapper = mount(DailySummaryViewer, {
      props: {
        summaryPath: '/test/path/summary.md'
      }
    })

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(invokeMock).toHaveBeenCalledWith('read_file', { path: '/test/path/summary.md' })
    expect(wrapper.text()).toContain(testContent)
  })

  it('displays file path', async () => {
    invokeMock.mockResolvedValue('Content')
    const testPath = '/test/path/summary.md'

    const wrapper = mount(DailySummaryViewer, {
      props: {
        summaryPath: testPath
      }
    })

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain(`文件路径: ${testPath}`)
  })

  it('shows error when path is empty', async () => {
    const wrapper = mount(DailySummaryViewer, {
      props: {
        summaryPath: ''
      }
    })

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('日报路径为空')
  })

  it('shows error when loading fails', async () => {
    invokeMock.mockRejectedValue(new Error('File not found'))

    const wrapper = mount(DailySummaryViewer, {
      props: {
        summaryPath: '/test/path/summary.md'
      }
    })

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 10))

    expect(wrapper.text()).toContain('加载失败')
  })

  it('emits close event when close button is clicked', async () => {
    invokeMock.mockResolvedValue('Content')

    const wrapper = mount(DailySummaryViewer, {
      props: {
        summaryPath: '/test/path/summary.md'
      }
    })

    const closeButton = wrapper.findAll('button').find(btn => btn.text() === '✕')
    await closeButton.trigger('click')

    expect(wrapper.emitted('close')).toBeTruthy()
    expect(wrapper.emitted('close').length).toBe(1)
  })

  it('emits close event when clicking backdrop', async () => {
    invokeMock.mockResolvedValue('Content')

    const wrapper = mount(DailySummaryViewer, {
      props: {
        summaryPath: '/test/path/summary.md'
      }
    })

    const backdrop = wrapper.find('.fixed')
    await backdrop.trigger('click.self')

    expect(wrapper.emitted('close')).toBeTruthy()
  })

  it('opens directory in Finder when button is clicked', async () => {
    invokeMock.mockResolvedValue('Content')
    openMock.mockResolvedValue()

    const wrapper = mount(DailySummaryViewer, {
      props: {
        summaryPath: '/test/path/to/summary.md'
      }
    })

    await wrapper.vm.$nextTick()

    const finderButton = wrapper.findAll('button').find(btn => btn.text().includes('在 Finder 中显示'))
    await finderButton.trigger('click')

    await wrapper.vm.$nextTick()

    expect(openMock).toHaveBeenCalledWith('/test/path/to')
  })

  it('handles error when opening directory fails', async () => {
    invokeMock.mockResolvedValue('Content')
    openMock.mockRejectedValue(new Error('Failed to open'))
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    const wrapper = mount(DailySummaryViewer, {
      props: {
        summaryPath: '/test/path/summary.md'
      }
    })

    await wrapper.vm.$nextTick()

    const finderButton = wrapper.findAll('button').find(btn => btn.text().includes('在 Finder 中显示'))
    await finderButton.trigger('click')

    await wrapper.vm.$nextTick()

    expect(consoleErrorSpy).toHaveBeenCalledWith('Failed to open in Finder:', expect.any(Error))
    consoleErrorSpy.mockRestore()
  })

  it('extracts directory path correctly for nested paths', async () => {
    invokeMock.mockResolvedValue('Content')
    openMock.mockResolvedValue()

    const wrapper = mount(DailySummaryViewer, {
      props: {
        summaryPath: '/very/deep/nested/path/to/file/summary.md'
      }
    })

    await wrapper.vm.$nextTick()

    const finderButton = wrapper.findAll('button').find(btn => btn.text().includes('在 Finder 中显示'))
    await finderButton.trigger('click')

    await wrapper.vm.$nextTick()

    expect(openMock).toHaveBeenCalledWith('/very/deep/nested/path/to/file')
  })
})
