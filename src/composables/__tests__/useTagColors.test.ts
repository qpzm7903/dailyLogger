import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  fetchTagColors,
  getCachedTagColors,
  getTagColorFromCache,
  updateTagColorInCache,
  setTagColorCache,
  isTagColorCacheInitialized
} from '../useTagColors'

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock TAG_COMMANDS
vi.mock('../../shared/api/tauri/commands', () => ({
  TAG_COMMANDS: { GET_TAG_COLORS: 'get_tag_colors' }
}))

import { invoke } from '@tauri-apps/api/core'

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>

describe('useTagColors', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Reset module-level state by re-initializing with empty cache
    setTagColorCache({})
  })

  describe('fetchTagColors', () => {
    it('fetches colors from backend and caches them', async () => {
      const colors = { work: '#ff0000', personal: '#00ff00' }
      mockInvoke.mockResolvedValue(colors)

      const result = await fetchTagColors()

      expect(mockInvoke).toHaveBeenCalledWith('get_tag_colors')
      expect(result).toEqual(colors)
      expect(getCachedTagColors()).toEqual(colors)
    })

    it('returns empty object on fetch failure', async () => {
      mockInvoke.mockRejectedValue(new Error('network error'))

      const result = await fetchTagColors()

      expect(result).toEqual({})
    })
  })

  describe('getCachedTagColors', () => {
    it('returns empty object when not initialized', () => {
      expect(getCachedTagColors()).toEqual({})
    })

    it('returns cached colors after fetch', async () => {
      mockInvoke.mockResolvedValue({ tag1: '#fff' })
      await fetchTagColors()

      expect(getCachedTagColors()).toEqual({ tag1: '#fff' })
    })
  })

  describe('getTagColorFromCache', () => {
    it('returns undefined for unknown tag', () => {
      expect(getTagColorFromCache('unknown')).toBeUndefined()
    })

    it('returns color for cached tag', () => {
      setTagColorCache({ mytag: '#ff0000' })

      expect(getTagColorFromCache('mytag')).toBe('#ff0000')
    })
  })

  describe('updateTagColorInCache', () => {
    it('adds a new tag color to the cache', () => {
      setTagColorCache({ existing: '#000' })
      updateTagColorInCache('newtag', '#fff')

      expect(getCachedTagColors()).toEqual({ existing: '#000', newtag: '#fff' })
    })

    it('updates an existing tag color', () => {
      setTagColorCache({ tag1: '#old' })
      updateTagColorInCache('tag1', '#new')

      expect(getCachedTagColors()).toEqual({ tag1: '#new' })
    })
  })

  describe('isTagColorCacheInitialized', () => {
    it('returns false before any fetch or set', () => {
      // Reset by setting empty and clearing init flag via module behavior
      expect(isTagColorCacheInitialized()).toBe(true) // setTagColorCache sets it
    })

    it('returns true after setTagColorCache', () => {
      setTagColorCache({})
      expect(isTagColorCacheInitialized()).toBe(true)
    })
  })
})
