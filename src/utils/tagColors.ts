/**
 * Unified tag color utilities
 *
 * This module provides consistent tag color styling across the application.
 * It replaces the hardcoded tagColors in App.vue with a centralized color system.
 */

import { PRESET_TAG_COLORS } from './tags'

/**
 * Tailwind CSS classes for each preset color (static variant - no hover)
 * Used for tag display in App.vue record list and filter buttons
 */
export const TAG_COLOR_CLASSES: Record<string, string> = {
  blue: 'bg-blue-500/20 text-blue-400',
  green: 'bg-green-500/20 text-green-400',
  yellow: 'bg-yellow-500/20 text-yellow-400',
  red: 'bg-red-500/20 text-red-400',
  purple: 'bg-purple-500/20 text-purple-400',
  pink: 'bg-pink-500/20 text-pink-400',
  cyan: 'bg-cyan-500/20 text-cyan-400',
  orange: 'bg-orange-500/20 text-orange-400',
  teal: 'bg-teal-500/20 text-teal-400',
  indigo: 'bg-indigo-500/20 text-indigo-400',
}

/**
 * Tailwind CSS classes for each preset color (interactive variant - with hover)
 * Used for TagCloud, TagFilter, TagInput components
 */
export const TAG_COLOR_CLASSES_INTERACTIVE: Record<string, string> = {
  blue: 'bg-blue-500/30 text-blue-300 hover:bg-blue-500/50',
  green: 'bg-green-500/30 text-green-300 hover:bg-green-500/50',
  yellow: 'bg-yellow-400/30 text-yellow-200 hover:bg-yellow-400/50',
  red: 'bg-red-500/30 text-red-300 hover:bg-red-500/50',
  purple: 'bg-purple-500/30 text-purple-300 hover:bg-purple-500/50',
  pink: 'bg-pink-500/30 text-pink-300 hover:bg-pink-500/50',
  cyan: 'bg-cyan-500/30 text-cyan-300 hover:bg-cyan-500/50',
  orange: 'bg-orange-500/30 text-orange-300 hover:bg-orange-500/50',
  teal: 'bg-teal-500/30 text-teal-300 hover:bg-teal-500/50',
  indigo: 'bg-indigo-500/30 text-indigo-300 hover:bg-indigo-500/50',
}

/**
 * Default color class for unknown tags
 */
export const DEFAULT_TAG_COLOR_CLASS = 'bg-gray-500/20 text-gray-400'
export const DEFAULT_TAG_COLOR_CLASS_INTERACTIVE = 'bg-gray-500/30 text-gray-300 hover:bg-gray-500/50'

/**
 * Mapping from default tag categories to their assigned colors
 * These are the AI-generated work classification tags
 */
const DEFAULT_CATEGORY_COLOR_MAP: Record<string, string> = {
  '开发': 'blue',
  '会议': 'purple',
  '写作': 'green',
  '学习': 'yellow',
  '研究': 'cyan',
  '沟通': 'orange',
  '规划': 'pink',
  '文档': 'indigo',
  '测试': 'red',
  '设计': 'teal',
}

/**
 * Get color name for a tag name
 * - If the tag is in DEFAULT_TAG_CATEGORIES, returns its assigned color
 * - Otherwise, generates a consistent color based on the tag name hash
 *
 * @param tagName - The tag name (e.g., "开发", "会议")
 * @returns The color name (e.g., "blue", "purple")
 */
export function getTagColorName(tagName: string): string {
  // Check if it's a default category with assigned color
  if (DEFAULT_CATEGORY_COLOR_MAP[tagName]) {
    return DEFAULT_CATEGORY_COLOR_MAP[tagName]
  }

  // Generate consistent color based on tag name hash
  const colors = PRESET_TAG_COLORS
  let hash = 0
  for (let i = 0; i < tagName.length; i++) {
    const char = tagName.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash = hash & hash // Convert to 32-bit integer
  }
  return colors[Math.abs(hash) % colors.length]
}

/**
 * Get Tailwind CSS classes for a tag (static variant)
 *
 * @param tagName - The tag name (e.g., "开发", "会议")
 * @returns Tailwind CSS classes for styling the tag
 */
export function getTagColorClass(tagName: string): string {
  const colorName = getTagColorName(tagName)
  return TAG_COLOR_CLASSES[colorName] || DEFAULT_TAG_COLOR_CLASS
}

/**
 * Get Tailwind CSS classes for a color name (static variant)
 *
 * @param colorName - The color name (e.g., "blue", "purple")
 * @returns Tailwind CSS classes for styling
 */
export function getColorClass(colorName: string): string {
  return TAG_COLOR_CLASSES[colorName] || DEFAULT_TAG_COLOR_CLASS
}

/**
 * Get Tailwind CSS classes for a color name (interactive variant with hover)
 *
 * @param colorName - The color name (e.g., "blue", "purple")
 * @returns Tailwind CSS classes for styling with hover effects
 */
export function getColorClassInteractive(colorName: string): string {
  return TAG_COLOR_CLASSES_INTERACTIVE[colorName] || DEFAULT_TAG_COLOR_CLASS_INTERACTIVE
}