# Plan: Add Manual Screenshot Trigger Button

## Objective
Add a manual screenshot trigger button to the Auto Perception card for easier testing.

## Steps

### 1. Add Rust command
- File: `src-tauri/src/auto_perception/mod.rs`
- Add `trigger_capture` command that calls `capture_and_store()` immediately

### 2. Register command
- File: `src-tauri/src/main.rs`
- Add `trigger_capture` to invoke_handler

### 3. Add frontend button
- File: `src/App.vue`
- Add `isCapturing` state
- Add `triggerCapture` function
- Add button in Auto Perception card

### 4. Rebuild app

## Files to modify
- `src-tauri/src/auto_perception/mod.rs`
- `src-tauri/src/main.rs`  
- `src/App.vue`
