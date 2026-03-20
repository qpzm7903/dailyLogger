/**
 * Tag-related constants and types
 *
 * These constants should match the backend values in src-tauri/src/memory_storage/tags.rs
 */

/**
 * Preset tag colors available for manual tags
 * Must match PRESET_TAG_COLORS in backend
 */
export const PRESET_TAG_COLORS: readonly string[] = [
  'blue', 'green', 'yellow', 'red', 'purple', 'pink', 'cyan', 'orange'
] as const

/**
 * Default tag categories for work classification (AI-generated tags)
 * Must match DEFAULT_TAG_CATEGORIES in backend
 */
export const DEFAULT_TAG_CATEGORIES: readonly string[] = [
  '开发', '会议', '写作', '学习', '研究', '沟通', '规划', '文档', '测试', '设计'
] as const

/**
 * Type for preset color names
 */
export type PresetColor = typeof PRESET_TAG_COLORS[number]