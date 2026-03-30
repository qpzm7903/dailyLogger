import { describe, it, expect, vi } from 'vitest'
import { useThumbnailCache } from '../useThumbnailCache'

describe('useThumbnailCache', () => {
  it('loads and caches a thumbnail', async () => {
    const { getThumbnail, getCacheSize } = useThumbnailCache()
    const loader = vi.fn().mockResolvedValue('data:image/png;base64,abc')

    const result = await getThumbnail('/path/to/img.png', loader)

    expect(result).toBe('data:image/png;base64,abc')
    expect(loader).toHaveBeenCalledOnce()
    expect(getCacheSize()).toBe(1)
  })

  it('returns cached thumbnail without calling loader again', async () => {
    const { getThumbnail, getCacheSize } = useThumbnailCache()
    const loader = vi.fn().mockResolvedValue('data:image/png;base64,abc')

    await getThumbnail('/path/to/img.png', loader)
    const result = await getThumbnail('/path/to/img.png', loader)

    expect(result).toBe('data:image/png;base64,abc')
    expect(loader).toHaveBeenCalledOnce()
    expect(getCacheSize()).toBe(1)
  })

  it('caches multiple thumbnails independently', async () => {
    const { getThumbnail, getCacheSize } = useThumbnailCache()
    const loader = vi.fn()
      .mockResolvedValueOnce('thumb1')
      .mockResolvedValueOnce('thumb2')

    await getThumbnail('/img1.png', loader)
    await getThumbnail('/img2.png', loader)

    expect(getCacheSize()).toBe(2)
  })

  it('evicts oldest entry when cache exceeds max size (LRU)', async () => {
    const { getThumbnail, hasThumbnail, getCacheSize } = useThumbnailCache()
    const loader = vi.fn().mockImplementation((path: string) =>
      Promise.resolve(`thumb-${path}`)
    )

    // Fill cache to max (100 items) with paths /0 ... /99
    for (let i = 0; i < 100; i++) {
      await getThumbnail(`/${i}`, loader)
    }
    expect(getCacheSize()).toBe(100)
    expect(hasThumbnail('/0')).toBe(true)

    // Access /0 to make it recently used (LRU promotion)
    await getThumbnail('/0', loader)

    // Add one more to trigger eviction — /1 should be evicted, not /0
    await getThumbnail('/100', loader)
    expect(getCacheSize()).toBe(100)
    expect(hasThumbnail('/0')).toBe(true) // kept due to LRU access
    expect(hasThumbnail('/1')).toBe(false) // evicted as oldest unused
    expect(hasThumbnail('/100')).toBe(true) // newly added
  })

  it('reports hasThumbnail correctly', async () => {
    const { getThumbnail, hasThumbnail } = useThumbnailCache()
    expect(hasThumbnail('/missing.png')).toBe(false)

    await getThumbnail('/exists.png', vi.fn().mockResolvedValue('data'))
    expect(hasThumbnail('/exists.png')).toBe(true)
  })

  it('clears all cached thumbnails', async () => {
    const { getThumbnail, clearCache, getCacheSize } = useThumbnailCache()
    await getThumbnail('/a.png', vi.fn().mockResolvedValue('a'))
    await getThumbnail('/b.png', vi.fn().mockResolvedValue('b'))
    expect(getCacheSize()).toBe(2)

    clearCache()
    expect(getCacheSize()).toBe(0)
  })
})
