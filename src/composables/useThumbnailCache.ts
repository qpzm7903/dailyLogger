/**
 * Thumbnail memory cache composable
 * Implements LRU-style cache for thumbnail images with configurable max size
 */

import { ref } from 'vue'

const MAX_CACHE_SIZE = 100

export function useThumbnailCache() {
  const cache = ref(new Map<string, string>())

  const getThumbnail = async (
    path: string,
    loader: (path: string) => Promise<string>
  ): Promise<string> => {
    // Check cache first
    if (cache.value.has(path)) {
      return cache.value.get(path)!
    }

    // Load thumbnail
    const thumbnail = await loader(path)

    // Evict oldest if at capacity
    if (cache.value.size >= MAX_CACHE_SIZE) {
      const oldestKey = cache.value.keys().next().value
      if (oldestKey) {
        cache.value.delete(oldestKey)
      }
    }

    // Store in cache
    cache.value.set(path, thumbnail)
    return thumbnail
  }

  const hasThumbnail = (path: string): boolean => {
    return cache.value.has(path)
  }

  const clearCache = () => {
    cache.value.clear()
  }

  const getCacheSize = (): number => {
    return cache.value.size
  }

  return {
    getThumbnail,
    hasThumbnail,
    clearCache,
    getCacheSize
  }
}
