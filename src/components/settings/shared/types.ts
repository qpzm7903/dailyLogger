/**
 * Shared types for Settings components
 */

export interface ModelInfo {
  context_window: number
  max_tokens?: number
}

export interface ConnectionTestResult {
  success: boolean
  message: string
  latency_ms?: number
}

export interface OllamaModel {
  name: string
  size?: string
  modified_at?: string
  size_vram?: number
}

export interface RunningModel {
  name: string
  size_vram?: number
}

export interface OllamaModelsResult {
  success: boolean
  models: OllamaModel[]
  model_details?: OllamaModel[]
  message?: string
}

export type SettingsTab = 'basic' | 'ai' | 'capture' | 'output'

/**
 * Check if the current endpoint is an Ollama endpoint
 */
export function isOllamaEndpoint(url: string): boolean {
  if (!url) return false
  const urlLower = url.toLowerCase()
  return (
    urlLower.includes('localhost:11434') ||
    urlLower.includes('127.0.0.1:11434') ||
    urlLower.includes(':11434/v1') ||
    urlLower.includes(':11434/')
  )
}

/**
 * Format model size for display
 */
export function formatModelSize(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
  } else if (bytes >= 1024 * 1024) {
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  } else {
    return (bytes / 1024).toFixed(1) + ' KB'
  }
}