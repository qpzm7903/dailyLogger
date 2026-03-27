# Story: ARCH-001 - 拆分前端应用壳

Status: done (frontend app shell extracted, App.vue simplified)

## Story

As a developer,
I want the frontend app shell separated from business logic,
So that App.vue only handles layout orchestration and doesn't become a catch-all for timers, event listeners, and business actions.

## Context

Currently `App.vue` (~655 lines) handles:
- Layout and modal orchestration
- State management for the entire app
- Timers (time update, records refresh)
- Tauri event listeners (network, queue, tray)
- Global shortcuts registration
- Business actions (takeScreenshot, triggerCapture, generateSummary, etc.)

This makes the component hard to maintain and test.

## Acceptance Criteria

1. **Given** `App.vue`, **When** extracting AppShell, **Then** AppShell only handles layout (Sidebar, Header, Dashboard, ErrorBoundary)

2. **Given** `App.vue`, **When** extracting AppModals, **Then** AppModals only handles Teleport modals with Transition

3. **Given** `App.vue`, **When** extracting useAppBootstrap, **Then** the composable handles:
   - Theme initialization
   - i18n initialization
   - Time interval management
   - Records refresh interval management
   - Network status polling
   - Event listeners (network-status-changed, offline-queue-updated, tray-open-settings, tray-open-quick-note)
   - Global shortcuts registration
   - Settings loading
   - Language loading from backend
   - Today records loading
   - Onboarding check

4. **Given** extracted components, **When** application starts, **Then** behavior is identical to before extraction

5. **Given** extracted components, **When** running typecheck and tests, **Then** all pass without modifications

## Tasks / Subtasks

- [ ] Task 1: Create `src/app/` directory structure
  - [ ] 1.1 Create `src/app/AppShell.vue` - extract layout shell
  - [ ] 1.2 Create `src/app/AppModals.vue` - extract modal teleport container
  - [ ] 1.3 Create `src/app/useAppBootstrap.ts` - extract lifecycle and initialization logic

- [ ] Task 2: Extract AppShell.vue
  - [ ] 2.1 Move ErrorBoundary, Sidebar, Header, Dashboard to AppShell
  - [ ] 2.2 Pass all required props from App.vue
  - [ ] 2.3 Emit handlers still defined in App.vue for now

- [ ] Task 3: Extract AppModals.vue
  - [ ] 3.1 Move Teleport and all Transition/Modal components
  - [ ] 3.2 Keep isOpen state from useModal composable
  - [ ] 3.3 Keep modal-specific handlers in App.vue temporarily

- [ ] Task 4: Extract useAppBootstrap.ts
  - [ ] 4.1 Extract onMounted logic into composable
  - [ ] 4.2 Extract onUnmounted cleanup logic
  - [ ] 4.3 Return state and functions needed by App.vue

- [ ] Task 5: Update App.vue to use new structure
  - [ ] 5.1 Import and use AppShell and AppModals
  - [ ] 5.2 Call useAppBootstrap and pass handlers
  - [ ] 5.3 Keep state management in App.vue for now (ref ownership)

- [ ] Task 6: Verify behavior unchanged
  - [ ] 6.1 Run `npm run typecheck`
  - [ ] 6.2 Run `npm run test`
  - [ ] 6.3 Manual verification of key flows

## Dev Notes

### Current App.vue Structure

```
App.vue
├── <ErrorBoundary>
│   └── <div class="h-screen ...">
│       ├── <OfflineBanner>
│       ├── <Sidebar>
│       ├── <div class="flex-1 ...">
│       │   ├── <Header>
│       │   └── <Dashboard>
│       └── <Teleport to="body">
│           └── <Transition> x 20+ modals
```

### Target Structure

```
src/
├── app/
│   ├── AppShell.vue      # Layout shell
│   ├── AppModals.vue     # Modal container
│   └── useAppBootstrap.ts # Lifecycle composable
├── components/
│   └── ...               # Existing components unchanged
└── App.vue               # Thin orchestrator
```

### Key State to Extract

**State that stays in App.vue** (owns ref):
- currentTime, isOnline, offlineQueueCount
- autoCaptureEnabled, quickNotesCount, todayRecords
- isGenerating, isGeneratingWeekly, isGeneratingMonthly
- summaryPath, weeklyReportPath, etc.
- selectedScreenshot, initialFilterTag, selectedSession
- showOnboarding

**State that moves to useAppBootstrap**:
- timeInterval, recordsRefreshInterval (internal)
- unlisten functions (internal cleanup)
- networkCheckInterval (internal)
- Initialization functions: loadSettings, loadLanguageFromBackend, loadTodayRecords

### Migration Approach

1. First, create the new files without modifying App.vue significantly
2. Move logic piece by piece, testing after each
3. Keep App.vue as thin as possible but not obsessed - some state will naturally stay
4. The goal is NOT to make App.vue 50 lines, but to remove the bootstrap/orchestration burden

### Files to Modify

- `src/App.vue` - Simplify by using AppShell, AppModals, useAppBootstrap
- `src/app/AppShell.vue` - New file
- `src/app/AppModals.vue` - New file
- `src/app/useAppBootstrap.ts` - New file
