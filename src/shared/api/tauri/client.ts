/**
 * Tauri IPC Client
 * Unified wrapper around @tauri-apps/api/core invoke
 * Provides type-safe command invocation with consistent error handling
 *
 * Note: This module re-exports invoke directly from @tauri-apps/api/core
 * to maintain compatibility with existing test mocks.
 * Command name constants are imported from ./commands to ensure type safety.
 */

import { invoke as baseInvoke } from '@tauri-apps/api/core'
import { COMMANDS } from './commands'

// Re-export invoke directly for test mock compatibility
export const invoke = baseInvoke

// Also re-export command constants for use in feature actions
export { COMMANDS }

export default invoke