//! Application state management conventions
//!
//! This module documents the global state management patterns used in DailyLogger.
//!
//! ## Types of State
//!
//! ### 1. Module-local State (Preferred)
//!
//! Most state should be contained within its specific module. These are private
//! to the module and accessed through module functions.
//!
//! Examples of module-local state:
//! - `memory_storage::DB_CONNECTION` - Database connection (module-private)
//! - `crypto::ENCRYPTION_KEY` - Encryption key (module-private)
//! - `work_time::WORK_TIME_LEARNER` - Work time patterns (module-private)
//! - `capture_service::SCREEN_STATE` - Capture state (module-private)
//! - `offline_queue::QUEUE_PROCESSING` - Queue processing flag (module-private)
//! - `silent_tracker::SILENT_PATTERN_TRACKER` - Silent pattern tracker (module-private)
//!
//! **Rule**: If state is only used within a single module, keep it module-local.
//!
//! ### 2. Application State (AppState in lib.rs)
//!
//! State that needs to be accessed from multiple modules should be placed in
//! `AppState` in `lib.rs`. This provides a central registry for cross-cutting state.
//!
//! Current AppState fields (defined in lib.rs):
//! - `auto_capture_running: bool` - Whether auto-capture is active
//!
//! **Rule**: Only add to AppState if the state genuinely needs to be accessed
//! by multiple top-level modules.
//!
//! ## Global State Anti-patterns
//!
//! ### Don't: Create new `Lazy<Mutex<...>>` at module level
//!
//! ```rust,ignore
//! // BAD - scattered global state
//! static MY_STATE: Lazy<Mutex<MyState>> = Lazy::new(|| Mutex::new(MyState::default()));
//! ```
//!
//! ### Do: Use module-local state with accessor functions
//!
//! ```rust,ignore
//! // GOOD - module-local state with controlled access
//! static MY_STATE: Lazy<Mutex<MyState>> = Lazy::new(|| Mutex::new(MyState::default()));
//!
//! pub fn get_my_state() -> Result<MutexGuard<'static, MyState>, String> {
//!     MY_STATE.lock().map_err(|e| format!("Lock error: {}", e))
//! }
//! ```
//!
//! ### Do: For cross-module state, add to AppState in lib.rs
//!
//! ```rust,ignore
//! // GOOD - central state management in lib.rs
//! #[derive(Default)]
//! pub struct AppState {
//!     pub my_cross_cutting_state: MyState,
//! }
//! ```
//!
//! ## Adding New Global State
//!
//! When adding new state that needs to persist across the application:
//!
//! 1. **First ask**: Does it really need to be global?
//!    - Can it be passed as a parameter?
//!    - Can it be scoped to a single module?
//!
//! 2. **If yes to global**:
//!    - If module-local: use `Lazy<Mutex<...>>` pattern with accessor functions
//!    - If cross-module: add to `AppState` in `lib.rs`
//!
//! 3. **Document** the state in this module with its purpose and usage
