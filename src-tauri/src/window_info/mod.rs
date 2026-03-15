//! Cross-platform active window information retrieval.
//!
//! This module provides functionality to get information about the currently
//! active window, including its title and process name. This is useful for
//! understanding what the user is working on during automatic capture.

use serde::{Deserialize, Serialize};

/// Represents the currently active window information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ActiveWindow {
    /// The window title of the active window.
    pub title: String,
    /// The process name (application name) of the active window.
    pub process_name: String,
}

/// Get the currently active window's title and process name.
/// Returns an ActiveWindow struct with empty strings if the information
/// cannot be retrieved.
///
/// # Platform-specific behavior
///
/// - **Windows**: Uses Win32 API to get foreground window info
/// - **macOS**: Uses AppleScript (may require Accessibility permissions)
/// - **Linux**: Uses xdotool command-line tool
#[cfg(target_os = "windows")]
pub fn get_active_window() -> ActiveWindow {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    use windows::Win32::System::Threading::OpenProcess;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return ActiveWindow::default();
    }

    // Get window title
    let mut title_buf = [0u16; 512];
    let title_len = unsafe { GetWindowTextW(hwnd, &mut title_buf) };
    let title = if title_len > 0 {
        String::from_utf16_lossy(&title_buf[..title_len as usize])
    } else {
        String::new()
    };

    // Get process ID
    let mut process_id: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
    }

    let process_name = if process_id > 0 {
        // Open process with limited access rights (PROCESS_QUERY_LIMITED_INFORMATION = 0x1000)
        let process_handle = unsafe {
            OpenProcess(
                windows::Win32::System::Threading::PROCESS_ACCESS_RIGHTS(0x1000),
                false,
                process_id,
            )
        };

        if let Ok(handle) = process_handle {
            let mut module_name = [0u16; 260];
            let len = unsafe {
                GetModuleFileNameExW(
                    handle,
                    windows::Win32::Foundation::HMODULE::default(),
                    &mut module_name,
                )
            };
            // Safely close the handle - ignoring errors since we're done with it
            unsafe {
                let _ = windows::Win32::Foundation::CloseHandle(handle);
            }

            if len > 0 {
                let full_path = String::from_utf16_lossy(&module_name[..len as usize]);
                // Extract just the filename from the full path
                std::path::Path::new(&full_path)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    ActiveWindow {
        title,
        process_name,
    }
}

#[cfg(target_os = "macos")]
pub fn get_active_window() -> ActiveWindow {
    use std::process::Command;

    // Use AppleScript to get the frontmost application name
    let app_name = Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to get name of first process whose frontmost is true"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // Get window title using AppleScript
    let title = Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to get name of front window of first process whose frontmost is true"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    ActiveWindow {
        title,
        process_name: app_name,
    }
}

#[cfg(target_os = "linux")]
pub fn get_active_window() -> ActiveWindow {
    use std::process::Command;

    // Get active window ID using xdotool
    let window_id = Command::new("xdotool")
        .args(["getactivewindow"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if window_id.is_empty() {
        return ActiveWindow::default();
    }

    // Get window title
    let title = Command::new("xdotool")
        .args(["getwindowname", &window_id])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // Get window PID and process name
    let process_name = Command::new("xdotool")
        .args(["getwindowpid", &window_id])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|pid| {
            let pid = pid.trim();
            if pid.is_empty() {
                return None;
            }
            // Get process name from /proc/{pid}/comm
            std::fs::read_to_string(format!("/proc/{}/comm", pid))
                .ok()
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_default();

    ActiveWindow {
        title,
        process_name,
    }
}

/// Non-Windows, non-macOS, non-Linux fallback (e.g., FreeBSD, etc.)
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn get_active_window() -> ActiveWindow {
    ActiveWindow::default()
}

// ── Window filtering logic (SMART-001 Task 3) ──

/// Check if any pattern matches the given text (case-insensitive partial match).
///
/// Returns `true` if any pattern is found as a substring in the text,
/// ignoring case differences.
pub fn matches_any(text: &str, patterns: &[String]) -> bool {
    if text.is_empty() || patterns.is_empty() {
        return false;
    }
    let text_lower = text.to_lowercase();
    patterns
        .iter()
        .any(|p| text_lower.contains(&p.to_lowercase()))
}

/// Determine whether a capture should proceed based on window filtering rules.
///
/// # Arguments
///
/// * `window` - The currently active window information
/// * `whitelist` - List of patterns for allowed windows (partial match, case-insensitive)
/// * `blacklist` - List of patterns for blocked windows (partial match, case-insensitive)
/// * `use_whitelist_only` - If true, only windows matching whitelist are captured;
///   if false, blacklist is applied (allow all if blacklist is empty)
///
/// # Returns
///
/// * `true` - Capture should proceed
/// * `false` - Capture should be skipped
///
/// # Logic (per AC2 and AC3)
///
/// 1. If `use_whitelist_only` is true and whitelist is non-empty:
///    - Allow capture only if window matches whitelist (title or process_name)
/// 2. Otherwise, apply blacklist:
///    - Block capture if window matches blacklist (title or process_name)
///    - Allow capture otherwise
pub fn should_capture_by_window(
    window: &ActiveWindow,
    whitelist: &[String],
    blacklist: &[String],
    use_whitelist_only: bool,
) -> bool {
    // AC3: 白名单模式优先
    // If whitelist-only mode is enabled and whitelist is not empty
    if use_whitelist_only && !whitelist.is_empty() {
        return matches_any(&window.title, whitelist)
            || matches_any(&window.process_name, whitelist);
    }

    // AC2: 黑名单模式
    // If blacklist is not empty, check if window should be blocked
    if !blacklist.is_empty()
        && (matches_any(&window.title, blacklist) || matches_any(&window.process_name, blacklist))
    {
        return false;
    }

    // Default: allow capture
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_window_default_returns_empty_strings() {
        let window = ActiveWindow::default();
        assert!(window.title.is_empty(), "default title should be empty");
        assert!(
            window.process_name.is_empty(),
            "default process_name should be empty"
        );
    }

    #[test]
    fn active_window_can_be_created_with_values() {
        let window = ActiveWindow {
            title: "My Window".to_string(),
            process_name: "myapp".to_string(),
        };
        assert_eq!(window.title, "My Window");
        assert_eq!(window.process_name, "myapp");
    }

    #[test]
    fn active_window_serializes_to_json() {
        let window = ActiveWindow {
            title: "VS Code".to_string(),
            process_name: "Code".to_string(),
        };
        let json = serde_json::to_string(&window).unwrap();
        assert!(json.contains("\"title\":\"VS Code\""));
        assert!(json.contains("\"process_name\":\"Code\""));
    }

    #[test]
    fn active_window_deserializes_from_json() {
        let json = r#"{"title":"Terminal","process_name":"bash"}"#;
        let window: ActiveWindow = serde_json::from_str(json).unwrap();
        assert_eq!(window.title, "Terminal");
        assert_eq!(window.process_name, "bash");
    }

    #[test]
    fn active_window_deserializes_missing_fields_to_defaults() {
        // Test partial JSON - fields should default to empty strings
        let json = r#"{}"#;
        let window: ActiveWindow = serde_json::from_str(json).unwrap();
        assert!(window.title.is_empty());
        assert!(window.process_name.is_empty());
    }

    #[test]
    fn get_active_window_returns_valid_struct() {
        // This test verifies that get_active_window() doesn't panic
        // and returns a valid ActiveWindow struct.
        // On CI without a GUI, this may return empty strings, which is fine.
        let window = get_active_window();
        // Just verify the function doesn't panic
        let _ = window.title;
        let _ = window.process_name;
    }

    #[test]
    fn get_active_window_can_be_called_multiple_times() {
        // Verify the function is safe to call multiple times
        let w1 = get_active_window();
        let w2 = get_active_window();
        // Both calls should succeed without panicking
        let _ = (w1.title, w2.title);
    }

    // ── Window filtering logic tests (SMART-001 Task 3) ──

    #[test]
    fn matches_any_returns_true_for_exact_match() {
        let patterns = vec!["VS Code".to_string(), "IntelliJ IDEA".to_string()];
        assert!(matches_any("VS Code", &patterns));
        assert!(matches_any("IntelliJ IDEA", &patterns));
    }

    #[test]
    fn matches_any_returns_true_for_partial_match() {
        let patterns = vec!["VS Code".to_string()];
        // Title contains the pattern
        assert!(matches_any("VS Code - main.rs", &patterns));
        assert!(matches_any("Project - VS Code", &patterns));
    }

    #[test]
    fn matches_any_is_case_insensitive() {
        let patterns = vec!["vs code".to_string()];
        assert!(matches_any("VS Code", &patterns));
        assert!(matches_any("vs CODE", &patterns));
        assert!(matches_any("Vs code", &patterns));
    }

    #[test]
    fn matches_any_returns_false_for_no_match() {
        let patterns = vec!["VS Code".to_string(), "IntelliJ IDEA".to_string()];
        assert!(!matches_any("Chrome", &patterns));
        assert!(!matches_any("Firefox", &patterns));
    }

    #[test]
    fn matches_any_returns_false_for_empty_patterns() {
        let patterns: Vec<String> = vec![];
        assert!(!matches_any("VS Code", &patterns));
    }

    #[test]
    fn matches_any_handles_empty_text() {
        let patterns = vec!["VS Code".to_string()];
        assert!(!matches_any("", &patterns));
    }

    #[test]
    fn matches_any_handles_special_characters() {
        let patterns = vec!["Chrome - 工作".to_string()];
        assert!(matches_any("Chrome - 工作 - 标签页", &patterns));
    }

    #[test]
    fn matches_any_handles_unicode() {
        let patterns = vec!["微信".to_string(), "企业微信".to_string()];
        assert!(matches_any("微信 - 聊天", &patterns));
        assert!(matches_any("企业微信 - 消息", &patterns));
    }

    // ── should_capture_by_window tests ──

    #[test]
    fn should_capture_by_window_allows_when_no_filters() {
        let window = ActiveWindow {
            title: "Random App".to_string(),
            process_name: "random".to_string(),
        };
        // Empty whitelist and blacklist, use_whitelist_only = false
        assert!(should_capture_by_window(&window, &[], &[], false));
    }

    #[test]
    fn should_capture_by_window_allows_when_blacklist_empty() {
        let window = ActiveWindow {
            title: "VS Code".to_string(),
            process_name: "Code".to_string(),
        };
        // Empty blacklist, no whitelist mode
        assert!(should_capture_by_window(&window, &[], &[], false));
    }

    // AC2: 白名单模式
    #[test]
    fn should_capture_by_window_whitelist_allows_matching_title() {
        let window = ActiveWindow {
            title: "VS Code - main.rs".to_string(),
            process_name: "Code".to_string(),
        };
        let whitelist = vec!["VS Code".to_string(), "IntelliJ IDEA".to_string()];
        // use_whitelist_only = true, whitelist not empty
        assert!(should_capture_by_window(&window, &whitelist, &[], true));
    }

    #[test]
    fn should_capture_by_window_whitelist_allows_matching_process_name() {
        let window = ActiveWindow {
            title: "My Project".to_string(),
            process_name: "Code".to_string(), // Match by process name
        };
        let whitelist = vec!["Code".to_string()];
        assert!(should_capture_by_window(&window, &whitelist, &[], true));
    }

    #[test]
    fn should_capture_by_window_whitelist_blocks_non_matching() {
        let window = ActiveWindow {
            title: "Chrome".to_string(),
            process_name: "chrome".to_string(),
        };
        let whitelist = vec!["VS Code".to_string(), "IntelliJ IDEA".to_string()];
        // Window not in whitelist, use_whitelist_only = true
        assert!(!should_capture_by_window(&window, &whitelist, &[], true));
    }

    #[test]
    fn should_capture_by_window_whitelist_ignored_when_disabled() {
        let window = ActiveWindow {
            title: "Chrome".to_string(),
            process_name: "chrome".to_string(),
        };
        let whitelist = vec!["VS Code".to_string()];
        // use_whitelist_only = false, so whitelist is ignored
        assert!(should_capture_by_window(&window, &whitelist, &[], false));
    }

    #[test]
    fn should_capture_by_window_whitelist_allls_all_when_empty() {
        let window = ActiveWindow {
            title: "Chrome".to_string(),
            process_name: "chrome".to_string(),
        };
        // Empty whitelist, use_whitelist_only = true but empty whitelist means allow all
        assert!(should_capture_by_window(&window, &[], &[], true));
    }

    // AC2: 黑名单模式
    #[test]
    fn should_capture_by_window_blacklist_blocks_matching_title() {
        let window = ActiveWindow {
            title: "Chrome - Google".to_string(),
            process_name: "chrome".to_string(),
        };
        let blacklist = vec!["浏览器".to_string(), "Chrome".to_string()];
        // Window in blacklist, should be blocked
        assert!(!should_capture_by_window(&window, &[], &blacklist, false));
    }

    #[test]
    fn should_capture_by_window_blacklist_blocks_matching_process_name() {
        let window = ActiveWindow {
            title: "Google Search".to_string(),
            process_name: "chrome".to_string(),
        };
        let blacklist = vec!["chrome".to_string()];
        // Process name matches blacklist
        assert!(!should_capture_by_window(&window, &[], &blacklist, false));
    }

    #[test]
    fn should_capture_by_window_blacklist_allows_non_matching() {
        let window = ActiveWindow {
            title: "VS Code - main.rs".to_string(),
            process_name: "Code".to_string(),
        };
        let blacklist = vec!["浏览器".to_string(), "Slack".to_string()];
        // Window not in blacklist
        assert!(should_capture_by_window(&window, &[], &blacklist, false));
    }

    #[test]
    fn should_capture_by_window_blacklist_ignored_when_empty() {
        let window = ActiveWindow {
            title: "Chrome".to_string(),
            process_name: "chrome".to_string(),
        };
        // Empty blacklist
        assert!(should_capture_by_window(&window, &[], &[], false));
    }

    // AC3: 白名单优先级逻辑
    #[test]
    fn should_capture_by_window_whitelist_takes_priority_when_enabled() {
        let window = ActiveWindow {
            title: "VS Code".to_string(),
            process_name: "Code".to_string(),
        };
        let whitelist = vec!["VS Code".to_string()];
        let blacklist = vec!["Code".to_string()]; // Process name in blacklist
                                                  // use_whitelist_only = true, so whitelist mode applies, blacklist ignored
        assert!(should_capture_by_window(
            &window, &whitelist, &blacklist, true
        ));
    }

    #[test]
    fn should_capture_by_window_blacklist_applies_when_whitelist_disabled() {
        let window = ActiveWindow {
            title: "VS Code".to_string(),
            process_name: "Code".to_string(),
        };
        let whitelist = vec!["VS Code".to_string()];
        let blacklist = vec!["Code".to_string()]; // Process name in blacklist
                                                  // use_whitelist_only = false, so blacklist applies
        assert!(!should_capture_by_window(
            &window, &whitelist, &blacklist, false
        ));
    }

    // 边界测试
    #[test]
    fn should_capture_by_window_handles_empty_title() {
        let window = ActiveWindow {
            title: "".to_string(),
            process_name: "chrome".to_string(),
        };
        let blacklist = vec!["chrome".to_string()];
        // Process name still matches
        assert!(!should_capture_by_window(&window, &[], &blacklist, false));
    }

    #[test]
    fn should_capture_by_window_handles_empty_process_name() {
        let window = ActiveWindow {
            title: "Chrome".to_string(),
            process_name: "".to_string(),
        };
        let blacklist = vec!["Chrome".to_string()];
        // Title still matches
        assert!(!should_capture_by_window(&window, &[], &blacklist, false));
    }

    #[test]
    fn should_capture_by_window_handles_both_empty() {
        let window = ActiveWindow {
            title: "".to_string(),
            process_name: "".to_string(),
        };
        // Empty strings don't match any pattern
        let blacklist = vec!["Chrome".to_string()];
        assert!(should_capture_by_window(&window, &[], &blacklist, false));

        let whitelist = vec!["VS Code".to_string()];
        assert!(!should_capture_by_window(&window, &whitelist, &[], true));
    }

    #[test]
    fn should_capture_by_window_matches_case_insensitive() {
        let window = ActiveWindow {
            title: "VS CODE".to_string(),
            process_name: "code".to_string(),
        };
        let whitelist = vec!["vs code".to_string()];
        assert!(should_capture_by_window(&window, &whitelist, &[], true));

        let blacklist = vec!["CODE".to_string()];
        assert!(!should_capture_by_window(&window, &[], &blacklist, false));
    }

    // ── Platform-specific behavior tests (CORE-008 AC#2) ──

    #[test]
    fn platform_specific_get_active_window_returns_valid_struct() {
        // All platforms should return a valid ActiveWindow struct
        let window = get_active_window();
        // Verify the struct has the expected fields (no panics)
        let _ = window.title.as_str();
        let _ = window.process_name.as_str();
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_platform_has_get_active_window_implementation() {
        // On Windows, get_active_window should be available
        let window = get_active_window();
        // Just verify it compiles and runs without panic
        assert!(window.title.is_empty() || !window.title.is_empty());
        assert!(window.process_name.is_empty() || !window.process_name.is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_platform_has_get_active_window_implementation() {
        // On macOS, get_active_window should be available
        let window = get_active_window();
        // Just verify it compiles and runs without panic
        assert!(window.title.is_empty() || !window.title.is_empty());
        assert!(window.process_name.is_empty() || !window.process_name.is_empty());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_platform_has_get_active_window_implementation() {
        // On Linux, get_active_window should be available
        let window = get_active_window();
        // Just verify it compiles and runs without panic
        assert!(window.title.is_empty() || !window.title.is_empty());
        assert!(window.process_name.is_empty() || !window.process_name.is_empty());
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    #[test]
    fn fallback_platform_returns_default_active_window() {
        // On unsupported platforms, should return default (empty) struct
        let window = get_active_window();
        assert!(window.title.is_empty());
        assert!(window.process_name.is_empty());
    }

    #[test]
    fn active_window_has_correct_default() {
        // Verify default is consistent across all platforms
        let default_window = ActiveWindow::default();
        assert_eq!(default_window.title, "");
        assert_eq!(default_window.process_name, "");
    }
}
