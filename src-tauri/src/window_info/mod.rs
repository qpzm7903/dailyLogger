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
    use std::mem::size_of;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    use windows::Win32::System::Threading::OpenProcess;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0 == 0 {
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
            unsafe {
                let _ = windows::Win32::Foundation::CloseHandle(handle.0);
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
}
