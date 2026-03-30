//! Infrastructure module - shared technical foundations
//!
//! This module contains cross-cutting technical concerns that are used
//! across multiple parts of the application.
//!
//! ## Modules
//!
//! - `retry` - Shared retry utilities (backoff, jitter, error classification)
//! - `state` - Application state management conventions and AppState definition

pub mod retry;
pub mod state;
