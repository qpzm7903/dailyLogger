/**
 * Tag color management composable
 *
 * Fetches and caches tag colors from the backend.
 * This replaces the hardcoded color mapping in tagColors.ts.
 */

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { TAG_COMMANDS } from '../shared/api/tauri/commands'

// Cache for tag colors fetched from backend
const tagColorCache = ref<Record<string, string>>({})
let isInitialized = false

/**
 * Fetch all tag colors from the backend
 */
export async function fetchTagColors(): Promise<Record<string, string>> {
  try {
    const colors = await invoke<Record<string, string>>(TAG_COMMANDS.GET_TAG_COLORS)
    tagColorCache.value = colors
    isInitialized = true
    return colors
  } catch (error) {
    console.error('Failed to fetch tag colors:', error)
    return {}
  }
}

/**
 * Get the cached tag colors (synchronous)
 * Returns empty object if not yet initialized
 */
export function getCachedTagColors(): Record<string, string> {
  return tagColorCache.value
}

/**
 * Check if tag colors have been initialized
 */
export function isTagColorCacheInitialized(): boolean {
  return isInitialized
}

/**
 * Get the color for a specific tag
 * Returns undefined if not found in cache
 */
export function getTagColorFromCache(tagName: string): string | undefined {
  return tagColorCache.value[tagName]
}

/**
 * Update a single tag's color in the cache
 */
export function updateTagColorInCache(tagName: string, color: string): void {
  tagColorCache.value = {
    ...tagColorCache.value,
    [tagName]: color
  }
}

/**
 * Set the entire color cache (e.g., after a full refresh)
 */
export function setTagColorCache(colors: Record<string, string>): void {
  tagColorCache.value = colors
  isInitialized = true
}
