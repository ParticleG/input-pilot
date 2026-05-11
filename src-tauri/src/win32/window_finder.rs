use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, IsWindowVisible};
use windows::core::BOOL;
use crate::model::target_spec::{TargetSpec, TitleMatchMode};
use super::win32_util::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowMatch {
    pub handle: isize, // HWND as isize for serialization
    pub title: String,
    pub class_name: String,
    pub process_name: String,
}

fn matches_title(target: &TargetSpec, title: &str) -> bool {
    if target.title_match_mode == TitleMatchMode::Ignore || target.window_title.is_empty() {
        return true;
    }
    if target.title_match_mode == TitleMatchMode::Exact {
        return title == target.window_title;
    }
    // Contains
    title.contains(&target.window_title)
}

fn matches_window(target: &TargetSpec, hwnd: HWND) -> bool {
    if target.visible_only {
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() {
                return false;
            }
        }
    }

    let class_name = get_window_class(hwnd);
    if !target.window_class.is_empty() && class_name != target.window_class {
        return false;
    }

    let process_name = crate::util::string_util::to_upper_ascii(&get_process_name(hwnd));
    if !target.process_name.is_empty()
        && process_name != crate::util::string_util::to_upper_ascii(&target.process_name)
    {
        return false;
    }

    matches_title(target, &get_window_text(hwnd))
}

fn build_window_match(hwnd: HWND) -> WindowMatch {
    WindowMatch {
        handle: hwnd.0 as isize,
        title: get_window_text(hwnd),
        class_name: get_window_class(hwnd),
        process_name: get_process_name(hwnd),
    }
}

/// Find first window matching the target spec.
pub fn find_first(target: &TargetSpec) -> Option<WindowMatch> {
    find_all(target).into_iter().next()
}

/// Find all windows matching the target spec.
pub fn find_all(target: &TargetSpec) -> Vec<WindowMatch> {
    let mut matches: Vec<WindowMatch> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_matched_windows_proc),
            LPARAM(&mut (target, &mut matches) as *mut (&TargetSpec, &mut Vec<WindowMatch>) as isize),
        );
    }
    matches
}

/// List all visible top-level windows.
pub fn list_visible_top_level_windows() -> Vec<WindowMatch> {
    let mut matches: Vec<WindowMatch> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_visible_windows_proc),
            LPARAM(&mut matches as *mut Vec<WindowMatch> as isize),
        );
    }
    matches
}

unsafe extern "system" fn enum_matched_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let context = &mut *(lparam.0 as *mut (&TargetSpec, &mut Vec<WindowMatch>));
    if matches_window(context.0, hwnd) {
        context.1.push(build_window_match(hwnd));
    }
    BOOL::from(true) // TRUE = continue enumeration
}

unsafe extern "system" fn enum_visible_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let matches = &mut *(lparam.0 as *mut Vec<WindowMatch>);
    if IsWindowVisible(hwnd).as_bool() {
        matches.push(build_window_match(hwnd));
    }
    BOOL::from(true)
}
