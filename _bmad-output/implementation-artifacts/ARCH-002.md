# Story: ARCH-002 - 建立统一 Tauri IPC Client

**Status**: in_progress (IPC client infrastructure created, core components updated)

## Story

As a developer,
I want all Tauri command invocations to go through a unified IPC client,
So that components don't scatter direct `invoke(...)` calls and command names are centralized.

## Context

Before this refactoring, Vue components called `invoke(...)` directly with hardcoded command strings:

```javascript
await invoke('get_settings')
await invoke('save_settings', { settings })
await invoke('trigger_capture')
```

This makes it difficult to:
- Find all usages of a command
- Change command names consistently
- Add logging or error handling uniformly
- Test command invocations

## Acceptance Criteria

1. **Given** any Vue component or composable, **When** it needs to call a Tauri command, **Then** it uses the unified IPC client or feature actions instead of direct `invoke(...)`

2. **Given** command names, **When** they are used across the codebase, **Then** they are referenced from centralized constants

3. **Given** feature actions, **When** they encapsulate business logic, **Then** components only handle UI state and event handling

## Implementation

### Created Files

```
src/shared/api/tauri/
├── commands.ts     # Centralized command name constants
└── client.ts      # Unified invoke wrapper

src/features/
├── capture/actions.ts   # Capture: takeScreenshot, triggerCapture, etc.
├── reports/actions.ts    # Reports: generateDailySummary, etc.
├── settings/actions.ts    # Settings: getSettings, saveSettings, etc.
├── records/actions.ts    # Records: getTodayRecords, deleteRecord, etc.
├── sessions/actions.ts   # Sessions: analyzeSession, etc.
└── system/actions.ts     # System: checkNetworkStatus, etc.
```

### Updated Components

- `src/App.vue` - Now uses captureActions, reportActions, settingsActions, recordsActions, systemActions
- `src/app/AppModals.vue` - Uses addQuickNote from captureActions
- `src/components/SessionListModal.vue` - Uses sessionActions
- `src/components/ScreenshotModal.vue` - Uses captureActions and recordsActions

### Remaining Components (still have direct invoke calls)

- QuickNoteWindow.vue
- TagInput.vue
- SettingsModal.vue
- ExportModal.vue
- ErrorBoundary.vue
- settings/OutputSettings.vue
- settings/AISettings.vue
- settings/BasicSettings.vue
- OnboardingModal.vue
- SessionDetailView.vue
- HistoryViewer.vue
- BackupModal.vue
- TagCloud.vue

## Usage Example

Before:
```javascript
import { invoke } from '@tauri-apps/api/core'
await invoke('save_settings', { settings })
```

After:
```javascript
import { settingsActions } from '../features/settings/actions'
await settingsActions.saveSettings(settings)
```

Or using command constants:
```javascript
import { invoke } from '../shared/api/tauri/client'
import { SETTINGS_COMMANDS } from '../shared/api/tauri/commands'

await invoke(SETTINGS_COMMANDS.SAVE_SETTINGS, { settings })
```
